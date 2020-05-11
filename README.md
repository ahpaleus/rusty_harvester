# Summary
This is a fuzzer written in Rust language based on binary instrumentation from Intel PIN to obtain coverage on executables run under MacOS (10.15.4 - Catalina in my case).

## Building
You need to have rust/cargo ecosystem installed on your system and pin directory in $PATH. What is more, Intel PIN needs to have _System Integrity Protection_ disabled on MacOS (https://support.apple.com/en-us/HT204899).

## Usage
Having Rust, Intel PIN in your $PATH you run fuzzer with the following manner:
```sh
$ cargo run example.jpg ./fuzzed_binary
```

Directories:  
 - `crashes/` - directory with unique crashes (with the corresponding id based on PC, e.g. `SIGSEGV_PC_29498`)
 - `queue/` - queue with the files which created a new coverage to mutate


### Example output
```sh
$ cargo run example.jpg ./exif
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/fuzzer example.jpg ./exif`
Filename: example.jpg
Length of corpus file: 5958 bytes
Fuzz case	0	|        0.00 fuzz cases/second	|  0 crash (0 unique)	|  0 coverage
Fuzz case	1	|        0.35 fuzz cases/second	|  0 crash (0 unique)	|  487 coverage
Fuzz case	2	|        0.48 fuzz cases/second	|  0 crash (0 unique)	|  502 coverage
Fuzz case	3	|        0.54 fuzz cases/second	|  0 crash (0 unique)	|  523 coverage
Fuzz case	4	|        0.60 fuzz cases/second	|  0 crash (0 unique)	|  526 coverage
Fuzz case	5	|        0.63 fuzz cases/second	|  0 crash (0 unique)	|  533 coverage
Fuzz case	6	|        0.65 fuzz cases/second	|  0 crash (0 unique)	|  535 coverage
Fuzz case	7	|        0.66 fuzz cases/second	|  0 crash (0 unique)	|  535 coverage
Fuzz case	8	|        0.69 fuzz cases/second	|  0 crash (0 unique)	|  537 coverage
Fuzz case	9	|        0.69 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	10	|        0.71 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	11	|        0.72 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	12	|        0.73 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	13	|        0.74 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	14	|        0.74 fuzz cases/second	|  0 crash (0 unique)	|  542 coverage
Fuzz case	15	|        0.73 fuzz cases/second	|  0 crash (0 unique)	|  543 coverage
Fuzz case	16	|        0.73 fuzz cases/second	|  1 crash (1 unique)	|  543 coverage
```

## Aim of creation of this fuzzer
The aim of writing such a fuzzer was to:
 - [x] Learn Rust language
 - [x] Learn binary instrumentation with Intel PIN
 - [x] Get familiar with blackbox fuzzing and obtaining coverage
 	 - Coverage is obtained from Intel PIN module which observes control-flows and new branch taken
 - [x] Learn how to work with ASLR 
 	 - I was able to obtain Relative Virtual Address by subtracting _IMG_LowAddress()_ base within Intel PIN module

## Conclusions
Rust is a great language - safe & ultra fast.  
Unfortunately, running binary with Intel PIN to obtain coverage is not the best idea due to the performance.

## Bibliography
 - [https://www.youtube.com/user/gamozolabs](https://www.youtube.com/user/gamozolabs)
 - [https://www.instytutpwn.pl/wp-content/uploads/2016/11/%C5%9Aledzenie-%C5%9Bcie%C5%BCki-wykonania-procesu-dla-security-researchera-i-programisty-ROBERT-%C5%9AWI%C4%98CKI.pdf](https://www.instytutpwn.pl/wp-content/uploads/2016/11/%C5%9Aledzenie-%C5%9Bcie%C5%BCki-wykonania-procesu-dla-security-researchera-i-programisty-ROBERT-%C5%9AWI%C4%98CKI.pdf)
 - Practical Binary Analysis, Build Your Own Linux Tools for Binary Instrumentation, Analysis, and Disassembly
by Dennis Andriesse
