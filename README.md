# TLSH rust reimplementation introducing colored hashes

We reimplemented the [original TLSH](https://github.com/trendmicro/tlsh) algorithm in Rust. Our implementation currently
is the modified version of the default hash calculation parameters, namely, 1 byte checksum, 128 buckets that result in
a 70 character long hash in hexadecimal representation. Note, however, that our implementation supports an extra
security feature that we added in order to prevent a recent attack on the TLSH scheme[^1]. As a consequence, the hash
value that our implementation produces can contain an extra byte compared to the original TLSH hash value, so it is 36
bytes long (i.e., 72 characters long in hexadecimal form) instead of 35 bytes.

[^1]: Gábor Fuchs, Roland Nagy, Levente Buttyán, A Practical Attack on the TLSH Similarity Digest Scheme, In Proceedings
of the 18th International Conference on Availability, Reliability and Security (ARES 2023), Benevento, Italy, August
29 - September 1, 2023. DOI: 10.1145/3600160.3600173