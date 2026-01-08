此测试对照了原始java版本和rust版本在多线程并发处理事务时的性能差距，可以看到rust版本平均耗时只有java版本的1/5不到。测试代码在[这里](../tests/concurrency_tx.rs)

## java版程序执行情况
 执行结果统计 (按耗时从小到大)
|   Thread   | Duration |
| ---------- | -------- |
|  thread-26 |  96 ms
|  thread-46 |  99 ms
|  thread-76 |  100 ms
|  thread-66 |  101 ms
|  thread-92 |  101 ms
|  thread-42 |  103 ms
|  thread-82 |  103 ms
|  thread-67 |  104 ms
|  thread-37 |  105 ms
|  thread-69 |  105 ms
|  thread-91 |  107 ms
|  thread-100 |  107 ms
|  thread-64 |  109 ms
|  thread-83 |  109 ms
|  thread-78 |  109 ms
|  thread-56 |  110 ms
|  thread-53 |  110 ms
|  thread-88 |  110 ms
|  thread-87 |  110 ms
|  thread-96 |  110 ms
|  thread-51 |  111 ms
|  thread-63 |  111 ms
|  thread-90 |  111 ms
|  thread-99 |  111 ms
|  thread-93 |  112 ms
|  thread-84 |  112 ms
|  thread-98 |  112 ms
|  thread-80 |  113 ms
|  thread-85 |  113 ms
|  thread-23 |  114 ms
|  thread-43 |  114 ms
|  thread-57 |  114 ms
|  thread-61 |  114 ms
|  thread-75 |  114 ms
|  thread-70 |  114 ms
|  thread-73 |  114 ms
|  thread-97 |  114 ms
|  thread-94 |  114 ms
|  thread-89 |  114 ms
|  thread-86 |  114 ms
|  thread-95 |  114 ms
|  thread-7 |  115 ms
|  thread-9 |  115 ms
|  thread-14 |  115 ms
|  thread-20 |  115 ms
|  thread-33 |  115 ms
|  thread-55 |  115 ms
|  thread-71 |  115 ms
|  thread-65 |  115 ms
|  thread-62 |  115 ms
|  thread-79 |  115 ms
|  thread-81 |  115 ms
|  thread-11 |  116 ms
|  thread-10 |  116 ms
|  thread-54 |  116 ms
|  thread-52 |  116 ms
|  thread-60 |  116 ms
|  thread-77 |  116 ms
|  thread-50 |  117 ms
|  thread-58 |  117 ms
|  thread-68 |  117 ms
|  thread-72 |  117 ms
|  thread-74 |  117 ms
|  thread-39 |  118 ms
|  thread-48 |  118 ms
|  thread-45 |  118 ms
|  thread-59 |  118 ms
|  thread-16 |  119 ms
|  thread-38 |  119 ms
|  thread-30 |  119 ms
|  thread-25 |  119 ms
|  thread-31 |  119 ms
|  thread-27 |  119 ms
|  thread-28 |  119 ms
|  thread-15 |  119 ms
|  thread-21 |  119 ms
|  thread-12 |  120 ms
|  thread-1 |  120 ms
|  thread-5 |  120 ms
|  thread-32 |  120 ms
|  thread-41 |  120 ms
|  thread-19 |  120 ms
|  thread-29 |  120 ms
|  thread-22 |  120 ms
|  thread-49 |  120 ms
|  thread-44 |  120 ms
|  thread-3 |  121 ms
|  thread-36 |  121 ms
|  thread-18 |  121 ms
|  thread-17 |  121 ms
|  thread-24 |  121 ms
|  thread-47 |  121 ms
|  thread-4 |  122 ms
|  thread-2 |  122 ms
|  thread-8 |  122 ms
|  thread-40 |  122 ms
|  thread-35 |  122 ms
|  thread-34 |  122 ms
|  thread-6 |  123 ms
|  thread-13 |  123 ms
------------------------------------
所有任务完成，总墙钟 123 ms  
平均 114.75 ms

## rust版程序执行情况
执行结果统计 (按耗时从小到大)
Thread ID    | Duration (ms)
| ----------- | ----------- |
79           | 11.60ms    | 
76           | 13.54ms    | 
55           | 14.66ms    | 
42           | 17.17ms    | 
90           | 17.64ms    | 
18           | 18.03ms    | 
47           | 18.26ms    | 
31           | 18.30ms    | 
44           | 18.35ms    | 
98           | 18.35ms    | 
91           | 18.81ms    | 
87           | 18.81ms    | 
61           | 18.91ms    | 
80           | 19.05ms    | 
63           | 19.13ms    | 
32           | 19.33ms    | 
25           | 19.47ms    | 
6            | 19.59ms    | 
7            | 19.97ms    | 
1            | 20.08ms    | 
85           | 20.12ms    | 
99           | 20.14ms    | 
58           | 20.16ms    | 
17           | 20.29ms    | 
12           | 20.29ms    | 
11           | 20.35ms    | 
41           | 20.40ms    | 
27           | 20.50ms    | 
52           | 20.73ms    | 
54           | 20.75ms    | 
73           | 20.95ms    | 
62           | 20.96ms    | 
86           | 20.99ms    | 
8            | 21.12ms    | 
93           | 21.13ms    | 
13           | 21.31ms    | 
22           | 21.40ms    | 
23           | 21.42ms    | 
36           | 21.48ms    | 
38           | 21.52ms    | 
33           | 21.59ms    | 
28           | 21.59ms    | 
64           | 21.63ms    | 
48           | 21.64ms    | 
4            | 21.73ms    | 
68           | 21.74ms    | 
69           | 21.74ms    | 
65           | 21.80ms    | 
5            | 21.80ms    | 
74           | 21.83ms    | 
82           | 21.87ms    | 
95           | 22.14ms    | 
20           | 22.23ms    | 
21           | 22.35ms    | 
24           | 22.38ms    | 
50           | 22.39ms    | 
46           | 22.40ms    | 
35           | 22.44ms    | 
59           | 22.48ms    | 
77           | 22.51ms    | 
81           | 22.64ms    | 
56           | 22.64ms    | 
43           | 22.71ms    | 
57           | 22.71ms    | 
19           | 22.83ms    | 
71           | 23.04ms    | 
67           | 23.06ms    | 
66           | 23.06ms    | 
84           | 23.07ms    | 
49           | 23.08ms    | 
45           | 23.46ms    | 
37           | 23.58ms    | 
40           | 23.62ms    | 
75           | 23.64ms    | 
39           | 23.64ms    | 
30           | 23.65ms    | 
10           | 23.92ms    | 
97           | 23.94ms    | 
29           | 23.97ms    | 
88           | 24.13ms    | 
92           | 24.14ms    | 
2            | 24.16ms    | 
51           | 24.16ms    | 
3            | 24.16ms    | 
70           | 24.23ms    | 
83           | 24.23ms    | 
26           | 24.27ms    | 
34           | 24.29ms    | 
14           | 24.29ms    | 
9            | 24.34ms    | 
15           | 24.40ms    | 
60           | 24.42ms    | 
89           | 24.62ms    | 
94           | 24.66ms    | 
0            | 24.70ms    | 
78           | 24.70ms    | 
96           | 24.75ms    | 
72           | 24.77ms    | 
53           | 24.78ms    | 
16           | 24.84ms    | 
----------------------------------------
所有线程执行完毕。
平均执行时间: 21.71 ms