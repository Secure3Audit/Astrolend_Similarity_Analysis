# Astrolend_Similarity_Analysis

## Purpose

The primary objective of this analysis is to assess the code similarity between the **[source repo](#source)** and the **[target repo](#target)**.

Our engineering team has embarked on a detailed direct comparison of the code repositories. This examination aims to
accurately quantify and understand the extent of similarity.

Our analysis result can be found in the **[conclusion](#conclusion)**.

## Code Repos

#### Source

* https://github.com/mrgnlabs/marginfi-v2
    * commit hash: `d33e649e415c354cc2a1e3c49131725552d69ba0`
      * [marginfi-v2/programs/marginfi/src](https://github.com/mrgnlabs/marginfi-v2/tree/d33e649e415c354cc2a1e3c49131725552d69ba0/programs/marginfi/src)
      * [marginfi-v2/programs/brick/src/lib.rs](https://github.com/mrgnlabs/marginfi-v2/blob/d33e649e415c354cc2a1e3c49131725552d69ba0/programs/brick/src/lib.rs)

#### Target

* https://github.com/Umbra-Lab/astrolend
    * commit hash: `5f3661a0dac366b7d32e079b93ba15b92ae51600`
      * [astrolend/programs/astrolend/src](https://github.com/Umbra-Lab/astrolend/tree/5f3661a0dac366b7d32e079b93ba15b92ae51600/programs/astrolend/src)
      * [astrolend/programs/brick/src/lib.rs](https://github.com/Umbra-Lab/astrolend/blob/5f3661a0dac366b7d32e079b93ba15b92ae51600/programs/brick/src/lib.rs)

A copy of the codes has also been cloned into [source_code/](source_code/) and [target_code/](target_code/) for
reference.

## Conclusion

Based on our comparison, we conclude that the source and the target are: **Functionally Identical** .

The majority of changes are:
1. File renames
2. Variable and function renames
3. Remvoed debug level log

With proper execution, the Astrolend protocol should perform in very similar ways to marginfi-v2. Prior security assessments on source repo and commit can be a good reference. 

Readers of this analysis, based on your security requirements, should determine on your own whether or not to directly trust the prior or existing security assessments or audits on the source repo.

Original audit reports on source repo can be found at https://github.com/mrgnlabs/marginfi-v2/tree/main/audits on commit `0f710a46ed202077be7ae192f45a6338a3364ce1` and `516637404c2db54d41022b7deb9cfd627aa2a824`



## Methodology for Similarity Comparison

In our approach, we primarily employed the fundamental algorithm underlying the `diff` utility to ascertain the degree
of similarity at the string level between the source and target repos.

The `diff` utility, widely used in text comparison and analysis, operates on the principle of finding the longest common
subsequence (LCS) between two sets of data, typically text files:

1. Longest Common Subsequence (LCS): The LCS is the heart of the diff algorithm. It finds the longest sequence of
   characters that appear in the same order in both files. Unlike substrings, the characters in a subsequence are not
   required to occupy consecutive positions. The LCS serves as a baseline for understanding the similarities between two
   files.
2. Differences Identification: After determining the LCS, diff analyzes the sections of the files that don't form part
   of this subsequence. These segments are the differences or changes. The algorithm efficiently pinpoints where the two
   files diverge from the LCS, marking these as either additions, deletions, or modifications.

The screenshots are from Beyond Compare for better visualization.

Tools used:

* `diff` - https://ss64.com/osx/diff.html
* DiffMerge: https://sourcegear.com/diffmerge/
* Beyond Compare: https://www.scootersoftware.com/

## Comparison Detail

#### Diff Compare -> marginfi-v2/programs/marginfi/src

* Diff:
    * [source_code/marginfi-v2/programs/marginfi/src](source_code/marginfi-v2/programs/marginfi/src)
    * [target_code/astrolend/programs/astrolend/src](target_code/astrolend/programs/astrolend/src)
* Output: [./results/diff_marginfi_src.patch](./results/diff_marginfi_src.patch)

```bash
$ git diff b3c1c62402a64d29f4e3b3683866ae75c61407f4 36f39ef5eb5869df8c2887c79eb36f6ae3ba6eb1 > diff_marginfi_src.patch
```

Main changes:

1. File renames
2. Variable and function renames
3. Remvoed debug level log

Screenshot from Beyond Compare for Reference:

- ./source_code/marginfi-v2/programs/marginfi/src   
  
  ![lib.png](results/lib.png)
  ![borrow.png](results/borrow.png)
  ![configure.png](results/configure.png)
  ![transfer_authority.png](results/transfer_authority.png)
  ![withdraw.png](results/withdraw.png)
  ![lib3.png](results/lib3.png)
  ![marginfi_account.png](results/marginfi_account.png)

cloc compare

`$ cloc --git --by-file b3c1c62402a64d29f4e3b3683866ae75c61407f4 36f39ef5eb5869df8c2887c79eb36f6ae3ba6eb1 --include-ext=rs`
```
       30 text file.
       30 text file.

github.com/AlDanial/cloc v 1.90  T=0.31 s (12.9 files/s, 22973.9 lines/s)
----------------------------------------------------------------------------------------------------------------------------------
File                                                                                         blank        comment           code
----------------------------------------------------------------------------------------------------------------------------------
results/src/state/marginfi_group.rs
 same                                                                                            0            133           1222
 modified                                                                                        0              3             46
 added                                                                                           0             10              0
 removed                                                                                         0              0             10
results/src/state/marginfi_account.rs
 same                                                                                            0             97           1115
 modified                                                                                        0              1             95
 added                                                                                           0              5              0
 removed                                                                                         0              0              5
results/src/state/price.rs
 same                                                                                            0             79            930
 modified                                                                                        0              3             74
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/liquidate.rs
 same                                                                                            0             72            288
 modified                                                                                        0              1             35
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/collect_bank_fees.rs
 same                                                                                            0             10            263
 modified                                                                                        0              0             14
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/add_pool.rs
 same                                                                                            0             19            263
 modified                                                                                        0              0             13
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/configure_bank.rs
 same                                                                                            0              3            184
 modified                                                                                        0              0             19
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/lib.rs
 same                                                                                            0              5            123
 modified                                                                                        0              3             66
 added                                                                                           0              0              2
 removed                                                                                         1              0              0
results/src/instructions/marginfi_group/handle_bankruptcy.rs
 same                                                                                            0             11            158
 modified                                                                                        0              2             23
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/utils.rs
 same                                                                                            0             11            148
 modified                                                                                        0              0             10
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/withdraw.rs
 same                                                                                            0             10            123
 modified                                                                                        0              0             17
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/borrow.rs
 same                                                                                            0             11            114
 modified                                                                                        0              0             18
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/events.rs
 same                                                                                            0              1            114
 modified                                                                                        0              2             11
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/repay.rs
 same                                                                                            0              8            105
 modified                                                                                        0              0             17
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/emissions.rs
 same                                                                                            0              3            103
 modified                                                                                        0              0             18
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/deposit.rs
 same                                                                                            0              8             95
 modified                                                                                        0              0             16
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/flashloan.rs
 same                                                                                            0             12             71
 modified                                                                                        0              2             34
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/constants.rs
 same                                                                                            0             17            102
 modified                                                                                        0              0              1
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/errors.rs
 same                                                                                            0              0            100
 modified                                                                                        0              0              3
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/macros.rs
 same                                                                                            0              2             95
 modified                                                                                        0              0              3
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/configure.rs
 same                                                                                            0             23             61
 modified                                                                                        0              1             31
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/close_balance.rs
 same                                                                                            0              0             39
 modified                                                                                        0              0             13
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/initialize.rs
 same                                                                                            0              0             24
 modified                                                                                        0              0             16
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/transfer_authority.rs
 same                                                                                            0              3             18
 modified                                                                                        0              1             10
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/accrue_bank_interest.rs
 same                                                                                            0              0             21
 modified                                                                                        0              0              5
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_group/initialize.rs
 same                                                                                            0              0             16
 modified                                                                                        0              0             10
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/marginfi_account/close.rs
 same                                                                                            0              0             12
 modified                                                                                        0              0              8
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/prelude.rs
 same                                                                                            0              0              3
 modified                                                                                        0              0              3
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/instructions/mod.rs
 same                                                                                            0              0              0
 modified                                                                                        0              0              4
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
results/src/state/mod.rs
 same                                                                                            0              0              1
 modified                                                                                        0              0              2
 added                                                                                           0              0              0
 removed                                                                                         0              0              0
----------------------------------------------------------------------------------------------------------------------------------
SUM:
 same                                                                                            0            538           5911
 modified                                                                                        0             19            640
 added                                                                                           0             15              2
 removed                                                                                         1              0             15
----------------------------------------------------------------------------------------------------------------------------------
```






