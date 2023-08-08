/*
增加测试用例,
conv用例计算超时,移除,
还有sl等用例存在若干未知的bug
brain系列变差明显,
game系列变好明显,

*/
Testing 00_bitset1.s...
Timer@0056-0064: 0H-0M-0S-820220us
TOTAL: 0H-0M-0S-820220us
Passed 00_bitset1.s!
---------------
Testing 00_bitset2.s...
Timer@0056-0064: 0H-0M-1S-667603us
TOTAL: 0H-0M-1S-667603us
Passed 00_bitset2.s!
---------------
Testing 00_bitset3.s...
Timer@0056-0064: 0H-0M-2S-463899us
TOTAL: 0H-0M-2S-463899us
Passed 00_bitset3.s!
---------------
Testing 01_mm1.s...
Timer@0065-0084: 0H-0M-11S-618951us
TOTAL: 0H-0M-11S-618951us
Passed 01_mm1.s!
---------------
Testing 01_mm2.s...
Timer@0065-0084: 0H-0M-10S-435506us
TOTAL: 0H-0M-10S-435506us
Passed 01_mm2.s!
---------------
Testing 01_mm3.s...
Timer@0065-0084: 0H-0M-7S-845994us
TOTAL: 0H-0M-7S-845994us
Passed 01_mm3.s!
---------------
Testing 02_mv1.s...
Timer@0059-0067: 0H-0M-5S-653803us
TOTAL: 0H-0M-5S-653803us
Passed 02_mv1.s!
---------------
Testing 02_mv2.s...
Timer@0059-0067: 0H-0M-4S-774617us
TOTAL: 0H-0M-4S-774617us
Passed 02_mv2.s!
---------------
Testing 02_mv3.s...
Timer@0059-0067: 0H-0M-5S-448407us
TOTAL: 0H-0M-5S-448407us
Passed 02_mv3.s!
---------------
Testing 03_sort1.s...
Timer@0090-0102: 0H-0M-1S-781640us
TOTAL: 0H-0M-1S-781640us
Passed 03_sort1.s!
---------------
Testing 03_sort2.s...
Timer@0090-0102: 0H-0M-27S-148639us
TOTAL: 0H-0M-27S-148639us
Passed 03_sort2.s!
---------------
Testing 03_sort3.s...
Timer@0090-0102: 0H-0M-4S-867388us
TOTAL: 0H-0M-4S-867388us
Passed 03_sort3.s!
---------------
Testing 04_spmv1.s...
Timer@0039-0047: 0H-0M-11S-987668us
TOTAL: 0H-0M-11S-987668us
Passed 04_spmv1.s!
---------------
Testing 04_spmv2.s...
Timer@0039-0047: 0H-0M-6S-928537us
TOTAL: 0H-0M-6S-928537us
Passed 04_spmv2.s!
---------------
Testing 04_spmv3.s...
Timer@0039-0047: 0H-0M-5S-341205us
TOTAL: 0H-0M-5S-341205us
Passed 04_spmv3.s!
---------------
Testing brainfuck-bootstrap.s...
Timer@0116-0118: 0H-0M-16S-70484us
TOTAL: 0H-0M-16S-70484us
Passed brainfuck-bootstrap.s!
---------------
Testing brainfuck-mandelbrot-nerf.s...
Timer@0116-0118: 0H-0M-51S-448381us
TOTAL: 0H-0M-51S-448381us
Passed brainfuck-mandelbrot-nerf.s!
---------------
Testing brainfuck-pi-nerf.s...
Timer@0116-0118: 0H-0M-3S-20910us
TOTAL: 0H-0M-3S-20910us
Passed brainfuck-pi-nerf.s!
---------------
Testing crypto-1.s...
./run_test.sh: line 24: 36011 Segmentation fault      ./"${test_file}" < "./performance/${file%.*}.in" > "${out_file}"
Error in crypto-1.s!
139							      |	5: -1846340457 491560967 -1896191841 1934725712 816099460
							      >	0
---------------
Testing crypto-2.s...
./run_test.sh: line 24: 36024 Segmentation fault      ./"${test_file}" < "./performance/${file%.*}.in" > "${out_file}"
Error in crypto-2.s!
139							      |	5: -1396642099 -1816588045 -1195799673 565966198 -850103767
							      >	0
---------------
Testing crypto-3.s...
./run_test.sh: line 24: 36037 Segmentation fault      ./"${test_file}" < "./performance/${file%.*}.in" > "${out_file}"
Error in crypto-3.s!
139							      |	5: 767224695 -334833364 -1747822804 1001542503 1343700927
							      >	0
---------------
Testing dead-code-elimination-1.s...
Timer@100016-100032: 0H-0M-0S-2us
TOTAL: 0H-0M-0S-2us
Passed dead-code-elimination-1.s!
---------------
Testing dead-code-elimination-2.s...
Timer@100016-100032: 0H-0M-0S-132us
TOTAL: 0H-0M-0S-132us
Passed dead-code-elimination-2.s!
---------------
Testing dead-code-elimination-3.s...
Timer@100016-100032: 0H-0M-0S-13131us
TOTAL: 0H-0M-0S-13131us
Passed dead-code-elimination-3.s!
---------------
Testing fft0.s...
Timer@0060-0079: 0H-0M-15S-408809us
TOTAL: 0H-0M-15S-408809us
Passed fft0.s!
---------------
Testing fft1.s...
Timer@0060-0079: 0H-0M-33S-894037us
TOTAL: 0H-0M-33S-894037us
Passed fft1.s!
---------------
Testing fft2.s...
Timer@0060-0079: 0H-0M-31S-641976us
TOTAL: 0H-0M-31S-641976us
Passed fft2.s!
---------------
Testing floyd-0.s...
Timer@0062-0064: 0H-0M-0S-84us
TOTAL: 0H-0M-0S-84us
Passed floyd-0.s!
---------------
Testing floyd-1.s...
Timer@0062-0064: 0H-0M-0S-69474us
TOTAL: 0H-0M-0S-69474us
Passed floyd-1.s!
---------------
Testing floyd-2.s...
Timer@0062-0064: 0H-0M-30S-510830us
TOTAL: 0H-0M-30S-510830us
Error in floyd-2.s!
640000: 0 4 4 4 4 3 4 4 2 3 4 4 3 4 4 3 3 4 3 3 4 3 3 4 3 4 3 |	640000: 0 4 4 4 4 3 4 4 2 3 4 4 3 4 4 3 3 4 3 3 4 3 3 4 3 4 3
---------------
Testing gameoflife-gosper.s...
Timer@0095-0106: 0H-0M-22S-634845us
TOTAL: 0H-0M-22S-634845us
Passed gameoflife-gosper.s!
---------------
Testing gameoflife-oscillator.s...
Timer@0095-0106: 0H-0M-19S-916952us
TOTAL: 0H-0M-19S-916952us
Passed gameoflife-oscillator.s!
---------------
Testing gameoflife-p61glidergun.s...
Timer@0095-0106: 0H-0M-19S-824385us
TOTAL: 0H-0M-19S-824385us
Passed gameoflife-p61glidergun.s!
---------------
Testing hoist-1.s...
Timer@0121-0123: 0H-0M-0S-867us
TOTAL: 0H-0M-0S-867us
Passed hoist-1.s!
---------------
Testing hoist-2.s...
Timer@0121-0123: 0H-0M-0S-745616us
TOTAL: 0H-0M-0S-745616us
Passed hoist-2.s!
---------------
Testing hoist-3.s...
Timer@0121-0123: 0H-0M-8S-677017us
TOTAL: 0H-0M-8S-677017us
Passed hoist-3.s!
---------------
Testing instruction-combining-1.s...
Timer@10015-10030: 0H-0M-0S-2us
TOTAL: 0H-0M-0S-2us
Passed instruction-combining-1.s!
---------------
Testing instruction-combining-2.s...
Timer@10015-10030: 0H-0M-0S-285us
TOTAL: 0H-0M-0S-285us
Passed instruction-combining-2.s!
---------------
Testing instruction-combining-3.s...
Timer@10015-10030: 0H-0M-0S-263us
TOTAL: 0H-0M-0S-263us
Passed instruction-combining-3.s!
---------------
Testing integer-divide-optimization-1.s...
Timer@1016-1031: 0H-0M-0S-3us
TOTAL: 0H-0M-0S-3us
Passed integer-divide-optimization-1.s!
---------------
Testing integer-divide-optimization-2.s...
Timer@1016-1031: 0H-0M-0S-743us
TOTAL: 0H-0M-0S-743us
Passed integer-divide-optimization-2.s!
---------------
Testing integer-divide-optimization-3.s...
Timer@1016-1031: 0H-0M-0S-11918us
TOTAL: 0H-0M-0S-11918us
Passed integer-divide-optimization-3.s!
---------------
Testing median0.s...
Timer@0059-0061: 0H-0M-4S-766834us
TOTAL: 0H-0M-4S-766834us
Passed median0.s!
---------------
Testing median1.s...
Timer@0059-0061: 0H-0M-0S-2350us
TOTAL: 0H-0M-0S-2350us
Passed median1.s!
---------------
Testing median2.s...
Timer@0059-0061: 0H-0M-29S-865588us
TOTAL: 0H-0M-29S-865588us
Passed median2.s!
---------------
Testing shuffle0.s...
Timer@0078-0090: 0H-0M-9S-650249us
TOTAL: 0H-0M-9S-650249us
Passed shuffle0.s!
---------------
Testing shuffle1.s...
Timer@0078-0090: 0H-0M-13S-747950us
TOTAL: 0H-0M-13S-747950us
Passed shuffle1.s!
---------------
Testing shuffle2.s...
Timer@0078-0090: 0H-0M-7S-550564us
TOTAL: 0H-0M-7S-550564us
Passed shuffle2.s!
---------------
Testing sl1.s...
compiler_product/sl1.s: Assembler messages:
compiler_product/sl1.s: Fatal error: can't fill 256 bytes in section .data of /home/user/new/tmp/ccWIHY4w.o: 'No space left on device'
./run_test.sh: line 35: ./sl1exe.out: No such file or directory
Error in sl1.s!
127							      |	600: 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 
							      >	600: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	600: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	0
---------------
rm: cannot remove 'sl1exe.out': No such file or directory
Testing sl2.s...
compiler_product/sl2.s: Assembler messages:
compiler_product/sl2.s: Fatal error: can't fill 256 bytes in section .data of /home/user/new/tmp/ccFTOW7B.o: 'No space left on device'
./run_test.sh: line 35: ./sl2exe.out: No such file or directory
Error in sl2.s!
127							      |	400: 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 
							      >	400: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	400: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	0
---------------
rm: cannot remove 'sl2exe.out': No such file or directory
Testing sl3.s...
compiler_product/sl3.s: Assembler messages:
compiler_product/sl3.s: Fatal error: can't fill 256 bytes in section .data of /home/user/new/tmp/ccWn78lR.o: 'No space left on device'
./run_test.sh: line 35: ./sl3exe.out: No such file or directory
Error in sl3.s!
127							      |	300: 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 
							      >	300: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	300: 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22
							      >	0
---------------
rm: cannot remove 'sl3exe.out': No such file or directory
Testing stencil0.s...
Timer@0023-0059: 0H-0M-0S-145359us
TOTAL: 0H-0M-0S-145359us
Error in stencil0.s!
524288: 183 138 55 127 224 116 232 182 255 255 27 51 151 126  |	524288: 183 138 55 127 224 116 232 182 255 255 27 51 151 126 
0							      /	0
---------------
Testing stencil1.s...
Timer@0023-0059: 0H-0M-0S-365557us
TOTAL: 0H-0M-0S-365557us
Passed stencil1.s!
---------------
Testing transpose0.s...
Timer@0028-0047: 0H-0M-7S-132854us
TOTAL: 0H-0M-7S-132854us
Passed transpose0.s!
---------------
Testing transpose1.s...
Timer@0028-0047: 0H-0M-7S-159134us
TOTAL: 0H-0M-7S-159134us
Passed transpose1.s!
---------------
Testing transpose2.s...
Timer@0028-0047: 0H-0M-16S-663797us
TOTAL: 0H-0M-16S-663797us
Passed transpose2.s!
---------------

Test finished!
Total: 56 files, 48 passed, 8 failed.
