//! 法 998244353 上の形式的指数 (exp) を実装するモジュールである.

use super::super::convolution;
use super::super::modulo;

impl super::FPS {
    /// `x^degree` まで (含む) の形式的指数を計算する (疎密を自動選択する).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた指数系列.
    ///
    /// # Constraints
    /// - 定数項は 0 でなければならない.
    /// - `degree + 1 < 998244353` でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: 実行時に `exp_dense` または `exp_sparse` を選択する.
    /// - 空間計算量: 実行時に選択される実装に依存する.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![0]);
    /// let exp = fps.exp(3).unwrap();
    /// assert_eq!(1, exp.get(0));
    /// ```
    pub fn exp(&self, degree: usize) -> Option<Self> {
        // 計算対象の項数は `degree + 1` であり, 溢れは `None` として扱う.
        let target_len = degree.checked_add(1)?;
        if target_len >= modulo::M as usize {
            return None;
        }
        if target_len > convolution::MAX_NTT_LEN {
            return None;
        }
        // 疎な系列に対しては疎実装を選択し, それ以外は密実装を用いる.
        if self.should_use_sparse_exp(degree) {
            self.exp_sparse(degree)
        } else {
            self.exp_dense(degree)
        }
    }

    /// `exp` が疎実装を選択すべきかどうかを判定する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `bool`: 疎実装を選ぶなら `true`.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: O(N). N は `self.len()` である.
    /// - 空間計算量: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn exp` から呼び出される.
    /// ```
    fn should_use_sparse_exp(&self, degree: usize) -> bool {
        let target_len = degree + 1;
        if target_len <= 1 {
            return false;
        }

        // 疎実装は O(K * len) と見積もる. ここで K は `1..=degree` の非ゼロ項数である.
        // 密実装は NTT 長 `t` に対して O(t log2 t) と見積もる. `exp_dense` は doubling を行うため,
        // おおよそ log2(len) 回の畳み込みを行うとしてコストを見積もる.
        let non_zero_count = self
            .non_zero_terms_iter()
            .take_while(|(i, _)| *i <= degree)
            .filter(|(i, _)| *i != 0)
            .count();

        let mut steps = 0_u32;
        let mut current_len = 1_usize;
        while current_len < target_len {
            steps += 1;
            current_len <<= 1;
        }

        let t = (2 * target_len).next_power_of_two();
        let sparse_cost = (target_len as f32) * (non_zero_count as f32);
        let dense_cost = (steps as f32) * (t as f32) * (t as f32).log2();

        sparse_cost < dense_cost
    }

    /// `x^degree` まで (含む) の形式的指数を計算する (密な実装).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた指数系列.
    ///
    /// # Constraints
    /// - 定数項は 0 でなければならない.
    /// - `degree + 1 < 998244353` でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K). K は `degree + 1` である.
    /// - 空間計算量: O(K). K は結果の項数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![0]);
    /// let exp = fps.exp_dense(3).unwrap();
    /// assert_eq!(1, exp.get(0));
    /// ```
    pub fn exp_dense(&self, degree: usize) -> Option<Self> {
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx2") {
                return self.exp_dense_avx2(degree);
            }
        }

        self.exp_dense_scalar(degree)
    }

    /// 非 SIMD 実装により, `x^degree` まで (含む) の形式的指数を計算する.
    ///
    /// `res = exp(self)` を満たす系列 `res` を, Newton 法で精度を 2 倍ずつ増やしながら求める.
    /// 具体的には `res <- res * (1 + self - log(res))` を繰り返し適用する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた指数系列.
    ///
    /// # Constraints
    /// - 定数項は 0 でなければならない.
    /// - `degree + 1 < 998244353` でなければならない.
    /// - `degree + 1 <= convolution::MAX_NTT_LEN` でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K). K は `degree + 1` である.
    /// - 空間計算量: O(K). K は結果の項数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn exp_dense` から呼び出される.
    /// ```
    fn exp_dense_scalar(&self, degree: usize) -> Option<Self> {
        // exp の定義より, 入力の定数項は 0 でなければならない.
        if self.get(0) != 0 {
            return None;
        }
        let target_len_full = degree.checked_add(1)?;
        if target_len_full >= modulo::M as usize {
            return None;
        }
        if target_len_full > convolution::MAX_NTT_LEN {
            return None;
        }
        if target_len_full == 1 {
            return Some(Self { coeffs: vec![1] });
        }

        // 初期値 res = 1 (次数 0 まで正しい) から開始し, 項数を 2 倍ずつ増やす.
        let mut res = Self { coeffs: vec![1] };
        let mut current_len = 1;

        while current_len < target_len_full {
            // 次の反復では, `target_len` までの正しさを得る.
            let target_len = (current_len << 1).min(target_len_full);

            // self を `target_len` まで切り詰め, 更新式へ渡す.
            let mut truncated = Self {
                coeffs: self.coeffs.iter().cloned().take(target_len).collect(),
            };
            truncated.trim();

            // delta = self - log(res) を計算し, `1 + delta` を res に掛ける.
            let mut delta = truncated - res.log_dense(target_len - 1)?;
            if delta.is_zero() {
                // delta が 0 のとき, `1 + delta` は 1 である.
                delta = Self { coeffs: vec![1] };
            } else {
                // 定数項に 1 を足して `1 + delta` を構築する.
                delta.coeffs[0] = modulo::add(delta.coeffs[0], 1);
            }

            res *= delta;
            res.truncate(target_len);
            current_len = target_len;
        }

        res.truncate(target_len_full);
        Some(res)
    }

    /// AVX2 + Montgomery により, `x^degree` まで (含む) の形式的指数を計算する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた指数系列.
    ///
    /// # Constraints
    /// - AVX2 が利用可能な環境でのみ呼び出す.
    /// - 定数項は 0 でなければならない.
    /// - `degree + 1 < 998244353` でなければならない.
    /// - `degree + 1 <= convolution::MAX_NTT_LEN` でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート, および内部実装に依存する).
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K). K は `degree + 1` である.
    /// - 空間計算量: O(K). K は結果の項数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // AVX2 依存のため, 直接の使用例は省略する.
    /// ```
    #[cfg(target_arch = "x86_64")]
    fn exp_dense_avx2(&self, degree: usize) -> Option<Self> {
        use super::super::convolution_mont;

        debug_assert!(std::is_x86_feature_detected!("avx2"));

        if self.get(0) != 0 {
            return None;
        }
        let target_len_full = degree.checked_add(1)?;
        if target_len_full >= modulo::M as usize {
            return None;
        }
        if target_len_full > convolution::MAX_NTT_LEN {
            return None;
        }
        if target_len_full == 1 {
            return Some(Self { coeffs: vec![1] });
        }

        let n = target_len_full.next_power_of_two();
        if n > convolution::MAX_NTT_LEN {
            return None;
        }

        let one_mont = convolution_mont::standard_to_mont_scalar(1);

        let mut h_mont = vec![0_u32; n];
        for (i, v) in h_mont.iter_mut().enumerate().take(self.len().min(n)) {
            *v = self.get(i);
        }

        unsafe {
            convolution_mont::standard_to_mont(&mut h_mont);
        }

        // 反復内の係数操作で用いる `(i)` と `inv(i)` をまとめて参照する.
        //
        // `modulo::inv` をループ内で呼ぶと O(log MOD) が支配的になり得るため,
        // `MAX_NTT_LEN` までの逆元列を 1 度だけ前計算して再利用する.
        let num_mont = convolution_mont::num_mont_table();
        let inv_num_mont = convolution_mont::inv_num_mont_table();

        let mut m = 1_usize;

        let mut f_coeffs = vec![one_mont];
        let mut g_coeffs = vec![one_mont];
        let mut g_ntt = vec![one_mont];

        // 反復内で再利用するバッファを確保し, 不要な再割当を避ける.
        let mut f2_ntt = Vec::new();
        let mut g0_ntt = Vec::new();
        let mut g2_old_ntt = Vec::new();
        let mut p2_ntt = Vec::new();
        let mut q_ntt = Vec::new();
        let mut q_coeffs = Vec::new();
        let mut r_coeffs = Vec::new();
        let mut fp = Vec::new();
        let mut s = Vec::new();
        let mut g2_ntt = Vec::new();
        let mut s2_ntt = Vec::new();
        let mut t2_coeffs = Vec::new();
        let mut u = Vec::new();
        let mut u2_ntt = Vec::new();
        let mut v2_coeffs = Vec::new();

        while m < n {
            let two_m = 2 * m;
            debug_assert!(two_m.is_power_of_two());
            debug_assert!(two_m <= n);

            let lg_m = m.trailing_zeros() as usize;
            let lg_2m = two_m.trailing_zeros() as usize;
            let inv_m_mont = convolution_mont::inv_len_mont(lg_m);
            let inv_2m_mont = convolution_mont::inv_len_mont(lg_2m);

            // --- ブロック A: 逆数 g を精度 m へ更新する (Newton) ---
            f2_ntt.clear();
            f2_ntt.resize(two_m, 0);
            f2_ntt[..m].copy_from_slice(&f_coeffs);
            unsafe {
                convolution_mont::ntt_mont(&mut f2_ntt);
            }

            g0_ntt.clear();
            g0_ntt.extend_from_slice(&g_coeffs);
            let powers = convolution_mont::ntt_doubling_powers_mont(m);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut g0_ntt, powers);
                convolution_mont::ntt_mont(&mut g0_ntt);
            }

            g2_old_ntt.clear();
            g2_old_ntt.reserve(two_m);
            g2_old_ntt.extend_from_slice(&g_ntt);
            g2_old_ntt.extend_from_slice(&g0_ntt);

            p2_ntt.clear();
            p2_ntt.extend_from_slice(&g2_old_ntt);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut p2_ntt, &g2_old_ntt);
                convolution_mont::mul_pointwise_mont(&mut p2_ntt, &f2_ntt);
                convolution_mont::intt_mont(&mut p2_ntt);
                convolution_mont::mul_scalar_mont(&mut p2_ntt, inv_2m_mont);
            }

            debug_assert_eq!(m, g_coeffs.len());
            unsafe {
                convolution_mont::double_mont(&mut g_coeffs);
                convolution_mont::sub_assign_mont(&mut g_coeffs, &p2_ntt[..m]);
            }

            // --- ブロック B: q = h' mod x^(m-1) ---
            q_coeffs.clear();
            q_coeffs.resize(m, 0);
            q_coeffs[..(m - 1)].copy_from_slice(&h_mont[1..m]);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut q_coeffs, &num_mont[1..=m]);
            }

            // --- ブロック C: r = f*q mod (x^m - 1) ---
            q_ntt.clear();
            q_ntt.extend_from_slice(&q_coeffs);
            unsafe {
                convolution_mont::ntt_mont(&mut q_ntt);
            }

            r_coeffs.clear();
            r_coeffs.extend_from_slice(&q_ntt);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut r_coeffs, &f2_ntt[..m]);
                convolution_mont::intt_mont(&mut r_coeffs);
                convolution_mont::mul_scalar_mont(&mut r_coeffs, inv_m_mont);
            }

            // --- ブロック D: s = x*(f' - r) mod (x^m - 1) ---
            fp.clear();
            fp.resize(m, 0);
            fp[..(m - 1)].copy_from_slice(&f_coeffs[1..m]);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut fp, &num_mont[1..=m]);
            }
            unsafe {
                convolution_mont::sub_assign_mont(&mut fp, &r_coeffs);
            }

            s.clear();
            s.resize(m, 0);
            if m == 1 {
                s[0] = 0;
            } else {
                s[0] = fp[m - 1];
                s[1..].copy_from_slice(&fp[..(m - 1)]);
            }

            // --- ブロック E: t = g*s mod x^m, かつ DFT(g,2m) を確保する ---
            g2_ntt.clear();
            g2_ntt.resize(two_m, 0);
            g2_ntt[..m].copy_from_slice(&g_coeffs);
            unsafe {
                convolution_mont::ntt_mont(&mut g2_ntt);
            }

            s2_ntt.clear();
            s2_ntt.resize(two_m, 0);
            s2_ntt[..m].copy_from_slice(&s);
            unsafe {
                convolution_mont::ntt_mont(&mut s2_ntt);
            }

            t2_coeffs.clear();
            t2_coeffs.extend_from_slice(&g2_ntt);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut t2_coeffs, &s2_ntt);
                convolution_mont::intt_mont(&mut t2_coeffs);
                convolution_mont::mul_scalar_mont(&mut t2_coeffs, inv_2m_mont);
            }

            // --- ブロック F: u = HighHalf(h - ∫(x^(m-1) t)) ---
            //
            // w = ∫(Shift(t, m-1)) とすると, i=0..m-1 に対して
            // w[m+i] = t[i] / (m+i) が成り立つため, 2m 全体の構築は不要である.
            // ここでは u[i] = h[m+i] - t[i]/(m+i) を直接計算する.
            u.clear();
            u.resize(m, 0);
            u.copy_from_slice(&t2_coeffs[..m]);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut u, &inv_num_mont[m..two_m]);
            }
            unsafe {
                convolution_mont::rev_sub_assign_mont(&mut u, &h_mont[m..two_m]);
            }

            // --- ブロック G: 上半分だけ f を更新する ---
            u2_ntt.clear();
            u2_ntt.resize(two_m, 0);
            u2_ntt[..m].copy_from_slice(&u);
            unsafe {
                convolution_mont::ntt_mont(&mut u2_ntt);
            }

            v2_coeffs.clear();
            v2_coeffs.extend_from_slice(&u2_ntt);
            unsafe {
                convolution_mont::mul_pointwise_mont(&mut v2_coeffs, &f2_ntt);
                convolution_mont::intt_mont(&mut v2_coeffs);
                convolution_mont::mul_scalar_mont(&mut v2_coeffs, inv_2m_mont);
            }

            f_coeffs.resize(two_m, 0);
            f_coeffs[m..].copy_from_slice(&v2_coeffs[..m]);

            // 次の反復の入口条件を満たすため, g は上半分を 0 として保持する.
            g_coeffs.resize(two_m, 0);
            std::mem::swap(&mut g_ntt, &mut g2_ntt);

            m = two_m;
        }

        unsafe {
            convolution_mont::mont_to_standard(&mut f_coeffs);
        }
        f_coeffs.truncate(target_len_full);
        let mut res = Self { coeffs: f_coeffs };
        res.trim();
        Some(res)
    }

    /// 疎な形式的べき級数の形式的指数を `x^degree` まで (含む) 計算する.
    ///
    /// `F = exp(f)` が満たす `F' = F f'` を用い, `f'` の疎な表現を使って次数の
    /// 低い順に計算する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた指数系列.
    ///
    /// # Constraints
    /// - 定数項は 0 でなければならない.
    /// - `degree + 1 < 998244353` でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: O(K * degree). K は非ゼロ係数の個数である.
    /// - 空間計算量: O(K + degree).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![0, 1, 0, 2]);
    /// let exp_sparse = fps.exp_sparse(4).unwrap();
    /// let exp_dense = fps.exp(4).unwrap();
    /// assert_eq!(exp_dense, exp_sparse);
    /// ```
    pub fn exp_sparse(&self, degree: usize) -> Option<Self> {
        if self.get(0) != 0 {
            return None;
        }
        let target_len = degree.checked_add(1)?;
        if target_len >= modulo::M as usize {
            return None;
        }
        if target_len > convolution::MAX_NTT_LEN {
            return None;
        }
        if target_len == 1 {
            return Some(Self { coeffs: vec![1] });
        }

        let max_deriv_degree = degree - 1;

        // `f'(x)` の疎な表現を `max_deriv_degree` 以下で構築する.
        let mut sparse_deriv_terms = Vec::new();
        for (i, &c) in self.coeffs.iter().enumerate().skip(1) {
            if i > degree {
                break;
            }
            if c == 0 {
                continue;
            }
            let deg = i - 1;
            if deg > max_deriv_degree {
                break;
            }
            let coeff = modulo::mul(i as u32, c);
            sparse_deriv_terms.push((deg, coeff));
        }

        let inv_indices = modulo::build_inv_indices(target_len);

        let mut coeffs = vec![0_u32; target_len];
        coeffs[0] = 1;

        // 漸化式 (n + 1) * F_{n+1} = sum_{i=0}^n F_i * f'_{n-i} を用いる.
        for n in 0..degree {
            let mut acc = 0_u32;
            for &(deg, c) in sparse_deriv_terms.iter().take_while(|&&(deg, _)| deg <= n) {
                acc = modulo::add(acc, modulo::mul(coeffs[n - deg], c));
            }
            coeffs[n + 1] = modulo::mul(acc, inv_indices[n + 1]);
        }

        let mut res = Self { coeffs };
        res.trim();
        Some(res)
    }
}
