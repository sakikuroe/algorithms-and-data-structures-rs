const LOG_LEN: usize = 32;
 
#[derive(Clone)]
pub struct SuccinctBitVector {
    len: usize,
    bit: Vec<u16>,
    chunk: Vec<u32>,
    block: Vec<Vec<u16>>,
    table: Vec<u16>,
}
 
impl SuccinctBitVector {
    pub fn new(v: Vec<usize>) -> Self {
        let len = v.len();
        let table = (0_usize..1 << (LOG_LEN / 2))
            .map(|x| x.count_ones() as u16)
            .collect::<Vec<_>>();
 
        let cw = LOG_LEN * LOG_LEN;
        let bw = LOG_LEN / 2;
        let cnum = (len + cw - 1) / cw;
        let bnum = cw / bw;
 
        let mut bit = vec![0; cnum * bnum];
 
        let set = |bit: &mut Vec<u16>, i: usize, b: u8| {
            assert!(b <= 1);
            let bpos = i / bw;
            let offset = i % bw;
 
            if b == 0 {
                bit[bpos] &= !(1 << offset);
            } else {
                bit[bpos] |= 1 << offset;
            }
        };
 
        for i in 0..v.len() {
            set(&mut bit, i, v[i] as u8);
        }
 
        let mut chunk = vec![0_u32; cnum + 1];
        let mut block = vec![vec![0_u16; bnum]; cnum];
 
        for i in 0..cnum {
            block[i][0] = 0;
            for j in 0..bnum - 1 {
                block[i][j + 1] = block[i][j] + table[bit[i * bnum + j] as usize];
            }
            chunk[i + 1] = chunk[i]
                + block[i][bnum - 1] as u32
                + table[bit[(i + 1) * bnum - 1] as usize] as u32;
        }
 
        SuccinctBitVector {
            len,
            bit,
            chunk,
            block,
            table,
        }
    }
 
    pub fn len(&self) -> usize {
        self.len
    }
 
    pub fn access(&self, i: usize) -> usize {
        let bw = LOG_LEN / 2;
        let bpos = i / bw;
        let offset = i % bw;
        (self.bit[bpos] >> offset & 1) as usize
    }
 
    pub fn rank(&self, i: usize) -> usize {
        let cw = LOG_LEN * LOG_LEN;
        let bw = LOG_LEN / 2;
        let bnum = cw / bw;
 
        let cpos = i / cw;
        let bpos = i % cw / bw;
        let offset = i % bw;
 
        let masked = (self.bit[cpos * bnum + bpos]) & ((1 << offset) - 1);
        (self.chunk[cpos] + self.block[cpos][bpos] as u32 + self.table[masked as usize] as u32)
            as usize
    }
}
 