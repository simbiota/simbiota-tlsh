use crate::{
    tlsh::{EFF_BUCKETS, TLSH},
    util::{calc_lvalue, Pearson},
};

const WINDOW_SIZE: usize = 5;
const WINDOW_SIZE_M1: usize = WINDOW_SIZE - 1;

/// An error during TLSH calculation
#[derive(Copy, Clone, Debug)]
pub enum TLSHError {
    /// The data was too short or too long for TLSH calculation
    Length,
    /// The data did not have sufficient variety for TLSH calculation
    Variety,
}

struct Color {
    pearson: Pearson,
    a_bucket: [u32; 256],
    checksum: u8,
    sliding_window: [u8; WINDOW_SIZE],
    finalized: Option<Result<TLSH, TLSHError>>,
}

/// Calculates multiple different-color TLSH hashes of data
///
/// 1. Instantiate by the `new` method providing a slice of colors to calculate
/// 2. Call the `update` method with each chunk of data
/// 3. Call the `finalize` method
/// 4. Get the calculated TLSH hashes by `get_hashes`
///
/// # Examples
///
/// ```
/// use ::simbiota_tlsh::{TLSHBuilder, TLSHError};
/// let mut builder = TLSHBuilder::default();
/// const DATA: [u8; 64] = [b'A'; 64];
/// builder.update(&DATA[..40]);
/// builder.update(&DATA[40..62]);
/// builder.update(&DATA[62..]);
/// builder.finalize();
/// assert!(matches!(builder.get_hashes()[0], Err(TLSHError::Variety)));
/// ```
///
/// ```
/// use ::simbiota_tlsh::TLSHBuilder;
/// let mut builder = TLSHBuilder::default();
/// let data: Vec<u8> = (1..100).collect();
/// builder.update(&data);
/// builder.finalize();
/// assert!(builder.get_hashes()[0].is_ok());
/// ```
///
/// ```
/// use ::simbiota_tlsh::{TLSHBuilder, TLSHError};
/// let mut builder = TLSHBuilder::default();
/// let data: Vec<u8> = (1..49).collect();
/// builder.update(&data);
/// builder.finalize();
/// assert!(matches!(builder.get_hashes()[0], Err(TLSHError::Length)));
/// ```

macro_rules! j_n {
    ($n:expr,$color_index:expr,$j:expr,$i:expr, $self:expr, $data:expr) => {
        {
        let j_n = $self.rng_index($j.wrapping_sub($n));
        if $i > $n {
            $self.colors[$color_index].sliding_window[j_n] = $data[$i-$n];
        }
        j_n
        }
    };
}

macro_rules! a_buckets {
    ($self: expr, $n: expr, $p2: expr, $p3: expr, $p4:expr, $p5:expr, $p6:expr) => {
        {
            // WARNING: THIS IS WRONG FOR MULTIPLE COLORS
            // FIXME: Fix salt calculation
        let i_1 = $self.colors[$n].pearson.fast_b_mapping(49,$p2, $p3,$p4) as usize;
            $self.colors[$n].a_bucket[i_1] += 1;
        let i_2 = $self.colors[$n].pearson.fast_b_mapping(12,$p2,$p3,$p5) as usize;
            $self.colors[$n].a_bucket[i_2] += 1;
        let i_3 = $self.colors[$n].pearson.fast_b_mapping(178,$p2,$p4,$p5) as usize;
            $self.colors[$n].a_bucket[i_3] += 1;
        let i_4 = $self.colors[$n].pearson.fast_b_mapping(166,$p2,$p4,$p6) as usize;
            $self.colors[$n].a_bucket[i_4] += 1;
        let i_5 = $self.colors[$n].pearson.fast_b_mapping(84,$p2,$p3,$p6) as usize;
            $self.colors[$n].a_bucket[i_5] += 1;
        let i_6 = $self.colors[$n].pearson.fast_b_mapping(230,$p2,$p5,$p6) as usize;
            $self.colors[$n].a_bucket[i_6] += 1;
    }
    };
}


pub struct TLSHBuilder {
    colors: Vec<Color>,
    data_len: usize,
}

impl TLSHBuilder {
    /// Create an initialized instance of TLSHBuilder
    ///
    ///
    /// # Arguments
    ///
    /// * `colors` - Slice containing the color numbers of hashes to calculate
    pub fn new(colors: &[u8]) -> Self {
        Self {
            colors: colors
                .iter()
                .map(|v| Color {
                    pearson: Pearson::new(*v),
                    a_bucket: [0; 256],
                    checksum: 0,
                    finalized: None,
                    sliding_window: [0; WINDOW_SIZE],
                })
                .collect(),
            data_len: 0,
        }
    }

    #[allow(dead_code)]
    /// Clear the builder to reuse it to calculate the same hash colors of other data
    pub fn reset(&mut self) {
        for v in self.colors.iter_mut() {
            v.a_bucket = [0; 256];
            v.checksum = 0;
            v.finalized = None;
        }
        self.data_len = 0;
    }

    #[allow(dead_code)]
    fn update_window(&mut self, window: &[u8]) {
        let update_b_mapping = |c: &mut Color, s: u8, i: usize, j: usize, k: usize| {
            let h = c.pearson.fast_b_mapping(s, window[i], window[j], window[k]) as usize;
            if h < EFF_BUCKETS {
                c.a_bucket[h] += 1;
            }
        };
        for c in self.colors.iter_mut() {
            update_b_mapping(c, 2, 4, 3, 2);
            update_b_mapping(c, 3, 4, 3, 1);
            update_b_mapping(c, 5, 4, 2, 1);
            update_b_mapping(c, 7, 4, 2, 0);
            update_b_mapping(c, 11, 4, 3, 0);
            update_b_mapping(c, 13, 4, 1, 0);
            c.checksum = c
                .pearson
                .fast_b_mapping(0, window[4], window[3], c.checksum);
        }
    }

    fn rng_index(&self, index: usize) -> usize {
        (index.wrapping_add(WINDOW_SIZE)) % WINDOW_SIZE
    }

    fn fast_update(&mut self, data: &[u8]) {
        assert_eq!(WINDOW_SIZE, 5);
        let len = data.len();
        for n in 0..self.colors.len() {
            let mut j: usize = (self.data_len % (WINDOW_SIZE)) as i32 as usize;
            let mut fed_len = self.data_len;
            let mut checksum: u8 = self.colors[n].checksum;

            let mut i: usize = 0;
            while i < len {
                if fed_len >= (WINDOW_SIZE_M1) {
                    if (i >= 4) && (i + 5 < len) {
                        let a0 = data[i - 4];
                        let a1 = data[i - 3];
                        let a2 = data[i - 2];
                        let a3 = data[i - 1];
                        let a4 = data[i];
                        let a5 = data[i + 1];
                        let a6 = data[i + 2];
                        let a7 = data[i + 3];
                        let a8 = data[i + 4];

                        checksum = self.colors[n].pearson.fast_b_mapping(1, a4, a3, checksum);
                        a_buckets!(self,n,a4,a3,a2,a1,a0);

                        checksum = self.colors[n].pearson.fast_b_mapping(1, a5, a4, checksum);
                        a_buckets!(self,n, a5,a4,a3,a2,a1);

                        checksum = self.colors[n].pearson.fast_b_mapping(1, a6, a5, checksum);
                        a_buckets!(self,n, a6,a5,a4,a3,a2);

                        checksum = self.colors[n].pearson.fast_b_mapping(1, a7, a6, checksum);
                        a_buckets!(self,n, a7,a6,a5,a4,a3);

                        checksum = self.colors[n].pearson.fast_b_mapping(1, a8, a7, checksum);
                        a_buckets!(self,n, a8, a7,a6,a5,a4);

                        i += 5;
                        fed_len += 5;
                        j=self.rng_index(j+5);
                    } else {
                        self.colors[n].sliding_window[j] = data[i];
                        let j_1 = j_n!(1,n,j,i,self,data);
                        let j_2 = j_n!(2,n,j,i,self,data);
                        let j_3 = j_n!(3,n,j,i,self,data);
                        let j_4 = j_n!(4,n,j,i,self,data);

                        checksum = self.colors[n].pearson.fast_b_mapping(1, self.colors[n].sliding_window[j], self.colors[n].sliding_window[j_1], checksum);
                        a_buckets!(self,n, self.colors[n].sliding_window[j], self.colors[n].sliding_window[j_1], self.colors[n].sliding_window[j_2], self.colors[n].sliding_window[j_3], self.colors[n].sliding_window[j_4]);

                        i += 1;
                        fed_len += 1;
                        j = self.rng_index(j + 1);
                    }
                } else {
                    i += 1;
                    fed_len += 1;
                    j = self.rng_index(j + 1);
                }
            }
            self.colors[n].checksum = checksum;

        }
        self.data_len += len;

    }


    /// Add the next segment of data to process
    ///
    /// # Panics
    ///
    /// The method panics if adding less than 4 bytes on its first call (or first call after `clear`).
    pub fn update(&mut self, data: &[u8]) {
        for v in self.colors.iter_mut() {
            v.finalized = None;
        }
        self.fast_update(data);
    }

    pub fn fast_finalize(&mut self) {
        let lvalue = match calc_lvalue(self.data_len as u32) {
            Some(lv) => lv,
            None => {
                for v in self.colors.iter_mut() {
                    v.finalized = Some(Err(TLSHError::Length));
                }
                return;
            }
        };

        for n in 0..self.colors.len() {
            let (q1, q2, q3) = self.find_quartile(&self.colors[n].a_bucket);
            if q3 == 0 {
                self.colors[n].finalized = Some(Err(TLSHError::Variety));
                return;
            }

            let mut nonzero = 0;
            for i in 0..32 {
                for j in 0..4 {
                    if self.colors[n].a_bucket[4 * i + j] > 0 {
                        nonzero += 1;
                    }
                }
            }

            if nonzero <= 4 * 32 / 2 {
                self.colors[n].finalized = Some(Err(TLSHError::Variety));
                return;
            }
            let mut tlsh = TLSH{
                color: 0,
                checksum: 0,
                lvalue: 0,
                q_ratios: 0,
                codes: [0; 32],
            };
            for i in 0..32 {
                let mut h: u8 = 0;
                for j in 0..4 {
                    let k = self.colors[n].a_bucket[4 * i + j];
                    if q3 < k {
                        h += 3 << (j * 2);
                    }else if q2 < k{
                        h += 2 << (j * 2);
                    } else if q1 < k {
                        h += 1 << (j * 2);
                    }
                }
                tlsh.codes[i] = h;
            }

            tlsh.lvalue = lvalue;
            let q1r = (((q1 * 100) as f32) / (q3 as f32) % 16.0) as u8;
            let q2r = (((q2 * 100) as f32) / (q3 as f32) % 16.0) as u8;
            tlsh.q_ratios = (q1r << 4) | q2r;
            tlsh.checksum = self.colors[n].checksum;
            tlsh.color =self.colors[n].pearson.color;

            self.colors[n].finalized = Some(Ok(tlsh));
        }
    }

    fn find_quartile(&self, bucket: &[u32; 256]) -> (u32, u32, u32) {
        let mut bucket_copy:[u32; EFF_BUCKETS] = [0; EFF_BUCKETS];
        let mut short_cut_left:[u32; EFF_BUCKETS] = [0; EFF_BUCKETS];
        let mut short_cut_right:[u32; EFF_BUCKETS] = [0; EFF_BUCKETS];
        let mut spl = 0;
        let mut spr = 0;
        let p1 = EFF_BUCKETS / 4 - 1;
        let p2 = EFF_BUCKETS / 2 - 1;
        let p3 = EFF_BUCKETS - EFF_BUCKETS/4 - 1;
        let end = EFF_BUCKETS - 1;
        let mut q1: u32 = 0;
        let mut q2: u32 = 0;
        let mut q3: u32 = 0;

        bucket_copy[..=end].copy_from_slice(&bucket[..=end]);

        let mut l = 0;
        let mut r = end;
        loop {
            let ret = self.partition(&mut bucket_copy, l, r);
            if ret > p2 {
                r = ret - 1;
                short_cut_right[spr] = ret as u32;
                spr += 1;
            } else if ret < p2 {
                l = ret + 1;
                short_cut_left[spl] = ret as u32;
                spl += 1;
            } else {
                q2 = bucket_copy[p2];
                break;
            }
        }

        short_cut_left[spl] = (p2 - 1) as u32;
        short_cut_right[spr] = (p2 + 1) as u32;

        let mut l = 0;
        for i in 0..=spl {
            let mut r = short_cut_left[i] as usize;
            if r > p1 {
                loop {
                    let ret = self.partition( &mut bucket_copy, l, r);
                    if ret > p1 {
                        r = ret-1;
                    } else if ret < p1 {
                        l = ret+1;
                    } else {
                        q1 = bucket_copy[p1];
                        break;
                    }
                }
                break;
            } else if r < p1  {
                l = r;
            } else {
                q1 = bucket_copy[p1];
                break;
            }
        }

        let mut r = end;
        for i in 0..=spr {
            let mut l = short_cut_right[i] as usize;
            if l < p3  {
                loop {
                    let ret = self.partition( &mut bucket_copy, l, r );
                    if ret > p3 {
                        r = ret-1;
                    } else if ret < p3  {
                        l = ret+1;
                    } else {
                        q3 = bucket_copy[p3];
                        break;
                    }
                }
                break;
            } else if l > p3  {
                r = l;
            } else {
                q3 = bucket_copy[p3];
                break;
            }
        }

        (q1, q2, q3)
    }
    fn partition(&self, buf: &mut [u32; EFF_BUCKETS], left: usize, right: usize) -> usize {
        if left == right {
            return left;
        }

        if left + 1 == right {
            if buf[left] > buf[right] {
                buf.swap(left, right);
            }
            return left;
        }

        let mut ret = left;
        let pivot = (left + right) >> 1;

        let val = buf[pivot];
        buf[pivot] = buf[right];
        buf[right] = val;

        for i in left..right {
            if buf[i] < val {
                buf.swap(ret, i);
                ret += 1;
            }
        }
        buf[right] = buf[ret];
        buf[ret] = val;

        ret
    }


    /// Calculate the hashes of the processed data
    ///
    ///
    /// Call `finalize` before `get_hashes`
    pub fn finalize(&mut self) {
        self.fast_finalize();
    }

    /// Retreive the calculated TLSH hash objects
    ///
    /// # Panics
    ///
    /// The method panics if called without a `finalize` call since the last `update`.
    pub fn get_hashes(&self) -> Vec<Result<TLSH, TLSHError>> {
        self.colors
            .iter()
            .map(|v| v.finalized.expect("Calling get_hashes before finalize"))
            .collect()
    }

}

impl Default for TLSHBuilder {
    /// Create a `TLSHBuilder`, which only calculates the original TLSH hash of data
    ///
    ///
    /// Equivalent to `TLSHBuilder::new(&[0])`
    fn default() -> Self {
        Self::new(&[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral() {
        const DATA: [u8; 1000] = [b'Y'; 1000];
        let mut builder = TLSHBuilder::default();
        builder.update(&DATA);

        const RESULT: [u32; 256] = [0; 256];

        assert_eq!(RESULT, builder.colors[0].a_bucket);
    }

    #[test]
    fn test_69() {
        const DATA: [u8; 5] = [0x95, 0xf9, 0x32, 0xc1, 0x25];
        let mut builder = TLSHBuilder::default();
        builder.update(&DATA);

        let mut result = [0u32; 256];
        result[9] = 6;

        assert_eq!(result, builder.colors[0].a_bucket);
    }
}
