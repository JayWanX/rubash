# rubash GNU 兼容 Gap 清单（2026-09-14，权威口径）

口径：干净 HEAD `01253e7d` + TIMEOUT 150s + 路径归一化 + env 净化下的 **live A/B**
（rubash vs WSL GNU Bash 5.3.0）。结果 PASS=29 / DIFF=52 / TIMEOUT=2。
原始取证文件：`target/issue-suites/results/live/`（worktree）。
GNU 源码行号锚点基于 `third_party/bash/`（5.3.0 发行树）。

本文档是批量 remote issue 的依据；每条给：套件、现象、GNU 参考源码。

---

## 分类 A：平台/环境伪差异（12 项，不作为 rubash 代码缺陷）

| # | 套件 | 原因 | 处置 |
|---|------|------|------|
| A1 | glob-bracket | GNU 侧引用主仓库 `target/upstream-tests/examples/loadables/Makefile`，fixture 缺失 | 已随 harness 路径修复（1d07af34）消解 |
| A2 | rsh | `cp /bin/sh` 曾因 harness 未配置 `WINUXSH_ROOT` 而原样透传（见 G27 纠正记录）；fixture 根已接入后转换正常。剩余差异是 Git 的 /bin/sh 是 bash-as-sh、WSL 是 dash，消息不可能逐字节一致 | 转换已通；消息差异文档化豁免 |
| A3 | histexp | `/bin/sh: command not found`（平台）+ 缺 error prolog（归 G12） | 拆分 |
| A4 | coproc | `/etc/passwd` 在 Git Bash 不存在（WSL 有）→ `cat /etc/passwd` 失败 | 已由 harness fixture 根解决（`prepare_posix_root` 提供 `etc/passwd`） |
| A5 | extglob | `touch a:b`：NTFS 拒绝 `:`，Windows 无法创建 | 文档化豁免 |
| A6 | heredoc | `touch x*x`：NTFS 拒绝（os error 123） | 文档化豁免 |
| A7 | builtins | `setlocale en_US.UTF-8` Windows 无此 locale；`/var/tmp` 解释器场景 WSL 专属 | 部分归 G21 |
| A8 | invocation | `.: Is a directory` vs `Permission denied`（平台语义）；usage 缩进格式 | 文档化豁免 |
| A9 | intl | setlocale 警告 + glue 文件 CRLF（已随 1d07af34 消解大半）+ 1396 行 gettext 输出（rubash 无 i18n，长期项） | 文档化豁免 |
| A10 | glob | glue CRLF（已消解）+ locale 警告 + `AαB` argv Unicode 传递 | Unicode 部分归入后续 argv 编码专项 |
| A11 | procsub | glue 文件 `$'\r'`（已随 1d07af34 消解） | 复测确认 |
| A12 | posixexp | `/var/tmp/sh` 不存在；Windows 路径拼写已随 1d07af34 归一化 | 复测确认 |

## 分类 B：错误消息文本（归并为 G11/G12/G13/G14，见下）

`syntax error near syntax error near` 双重前缀、缺 `./script: line N:` prolog、
arith 措辞、heredoc EOF 警告行号、trap 参数校验、invalid identifier 校验。

## 分类 C：真 Gap（G1–G27）

### G1 set -e 退出语义
- 套件：`tests/set-e.tests`（GNU 74 行 / rubash 78 行）
- 现象：① `{ ...; false; }` 后 `$?` 应为 1，rubash 得 0（"after brace group failure: 0"）；
  ② `false || false`、`true && false`、`true && (exit 1)`、`true && true|false` 在
  `set -e` 下 rubash 未退出（多出 "no exit" 行）。
- GNU 源码：`execute_cmd.c` `exit_immediately_on_error` 判定链：655、777、877、1006、1013、1170、2913。

### G2 lastpipe 退出码
- 套件：`tests/lastpipe.tests`（23/18）
- 现象：`lastpipe1.sub returns 0`，GNU 返回 14；且 `AnB-e 1n2` 输出串行化错误。
- GNU 源码：`execute_cmd.c:2612-2760`（`uw_lastpipe_cleanup` 与 lastpipe 执行路径）。

### G3 globstar 目录遍历
- 套件：`tests/globstar.tests`（587/587）
- 现象：`**` 多展开 `c/aa c/ab`（GNU 在特定父目录组合下不进入 c/）。
- GNU 源码：`lib/glob/glob.c:1190,1216,1293,1502`（GX_GLOBSTAR 处理）。

### G4 test/内建判定错误
- 套件：`tests/test.tests`（339/334）、`tests/varenv.tests`（`[: : integer expression expected` 多打）
- 现象：多个 `test` 表达式返回 1，GNU 返回 0；`test.tests: line 118` 缺 "No such file or directory"。
- GNU 源码：`builtins/test.def`（一元/二元运算判定）。

### G5 declare -A/-ai 打印不回显赋值
- 套件：`tests/assoc.tests`（409/361）
- 现象：`declare -Ai chaff=([one]="10" [zero]="5" )` rubash 只打 `declare -Ai chaff`；
  整体元素/属性回显缺失（影响 array/quotearray）。
- GNU 源码：`builtins/declare.def`（declare 输出路径，attribute+值打印）。

### G6 nameref 间接展开
- 套件：`tests/nameref.tests`（588/560）
- 现象：`foo: invalid indirect expansion` 缺失；`unset` 传播顺序；`one=one != 2=4` 值错。
- GNU 源码：`subst.c:7913,7931,8107`（invalid indirect expansion）；`variables.c`（nameref cell）。

### G7 数组下标转义求值
- 套件：`tests/quotearray.tests`（152/128）、`tests/array.tests`（852/807）
- 现象：① `test=(first & second)` 应报 `syntax error near unexpected token '&'`；
  ② `'assoc[x\],b\[\$(echo uname >&2)]++'` 下标转义求值错（rubash 丢了转义文本）；
  ③ `bad array subscript` 缺失。
- GNU 源码：`arrayfunc.c:63`（bash_badsub_errmsg）；`subst.c` 下标展开与引号处理。

### G8 read 语义
- 套件：`tests/read.tests`（108/107）
- 现象：① `read -t` 超时后 `stat` 应为 2，rubash 为 1；
  ② 输入缓冲残留（`abcdefg|xyz` 混行）；
  ③ 高字节 `$'spring\375'` 处理。
- GNU 源码：`builtins/read.def:398-966`（超时返回路径 915/949）。

### G9 `$"..."` locale 引用与引号保留
- 套件：`tests/nquote.tests`（97/85）
- 现象：`$"world"` 应保留 `$"world"` 传给 argv（无 locale 时原样），rubash 展开为 `$world`；
  `'abcd'` 应保留引号输出。
- GNU 源码：`subst.c:4129`（`$'...'`/`$"..."` 展开）。

### G10 词切分中的反斜杠转义
- 套件：`tests/exp.tests`（549/533）、`tests/comsub.tests`（97/102）
- 现象：`\Hello world!\`（转义空格应保留 `\` 输出给 recho）、`\/tmp\/foo\/bar`、
  `")"` → `)\`、`a b` 两词被合并为一词。
- GNU 源码：`subst.c`（word_split/字段展开的转义保留）。

### G11 算术错误消息与求值
- 套件：`tests/arith.tests`、`tests/arith-for.tests`、`tests/cond.tests`、`tests/new-exp.tests`
- 现象：① "arithmetic syntax error: operand expected" 措辞（rubash 是 "syntax error"）；
  ② `let:`/`((:` 前缀；③ `division by 0`；④ `0#4: invalid number`；⑤ `jv += $iv` token 保留。
- GNU 源码：`expr.c:485,538,552,917,1120,1507`。

### G12 error prolog 与行号
- 套件：`tests/history.tests`、`tests/vredir.tests`、`tests/histexp.tests`、
  `tests/comsub2.tests`、`tests/printf.tests`、`tests/errors.tests`
- 现象：① 内建错误缺 `./script.tests: line N:` prolog（`history: -x: invalid option`）；
  ② 反向多打（vredir6 `redirection error` 前 rubash 多了 line 13——GNU 的
  redirection error prolog 不带行号，redir.c 走独立路径）；
  ③ 行号 off-by-one（comsub2 line 74 vs 75）。
- 已修（2026-09-14）：
  - trap 全部诊断走 error_prolog（`e372aa7b`，G13 覆盖）；
  - printf 7 处硬编码 `rubash: printf:` 前缀改走 diagnostic_prefix，
    `-v` identifier 消息反引号收尾改撇号（usage 行不带 prolog，
    与 printf.right 基线一致）；
  - unset（-x / -fv 同用）与 exec（invalid option / -a 缺参 / not found /
    spawn 失败）同样接入 prolog（`23f43397`）。
  - select 语义修正：invalid identifier 报错后中止 select、不再打印菜单
    （GNU errors.tests line 50-59 行为）；select 变量名保留 raw 词素
    （`select f\1` 报 `f\1'，此前被去转义成合法 f1 直接执行）。
- 剩余：history/vredir/histexp/comsub2 侧 prolog 与行号问题；
  errors.tests 中 declare readonly-function 族（line 76/85）与
  `unset: cannot simultaneously` 之外的 declare 选项差异。
- GNU 源码：`error.c:61-226`（error_prolog，print_lineno 语义）。

### G13 trap -p/-P 参数校验
- 套件：`tests/trap.tests`（145/140）
- 现象：`trap: cannot specify both -p and -P`、`trap: -P requires at least one signal name`
  两条错误缺失；EXIT trap 输出顺序 `+[8] false` 差异。
- GNU 源码：`builtins/trap.def:146,151`。

### G14 invalid identifier 校验
- 套件：`tests/errors.tests`（167/183）
- 现象：`` `1': not a valid identifier ``、`` `f\1': not a valid identifier ``、
  `` `invalid-name': not a valid identifier `` 在 unset/export/readonly 等内建中缺失。
- 已修（2026-09-14，`ee76c100`）：for/select 循环变量诊断去掉 `for:`/`select:`
  段、变量名保留 raw 词素（不做引号去除）。unset/export/readonly/declare/
  complete 的 invalid identifier 此前已实现且与 GNU 一致（`unset 1abc`
  在 GNU 5.3.0 静默，rubash 同）。
- GNU 源码：`builtins/common.c:210`；`error.c:461`；`execute_cmd.c:2379`。

### G15 heredoc/comsub EOF 警告
- 套件：`tests/comsub-eof.tests`、`tests/heredoc.tests`
- 现象：① comsub-eof0/3/4 的 "here-document delimited by end-of-file" 警告整体缺失；
  ② 警告行号偏移（line 2 vs line 1）；③ `unexpected end of file from '(' command on line 96` 措辞。
- GNU 源码：`make_cmd.c:627`（internal_warning here-document）。

### G16 xtrace 算术命令追踪
- 套件：`tests/set-x.tests`（72/49）
- 现象：`set -x` 下 rubash 不输出 `+ (( i=0 ))`、`+ (( i++ ))` 等算术命令追踪行。
- GNU 源码：`execute_cmd.c:3201,3940-3945`（xtrace_print_arith_cmd）。

### G17 重定向语义
- 套件：`tests/redir.tests`（181/173）、`tests/vredir.tests`
- 现象：① `to a`/`to b` 系列输出丢失（fd 复制/追加场景写错目标）；
  ② vredir8 line 30 缺 `No such file or directory`。
- GNU 源码：`redir.c`（do_redirection_internal）。

### G18 printf / attr 挂起（TIMEOUT）
- 套件：`tests/printf.tests`（GNU 395 行完成；rubash 输出 `[][]` 后卡死）、`tests/attr.tests`（偶发）
- 现象：rubash 侧 150s 不退出。
- GNU 源码：`builtins/printf.def:811`（invalid format character 路径附近）。
- 注：GNU-TIMEOUT.txt 已清空；printf 挂起属 rubash bug。

### G19 POSIX 模式语义
- 套件：`tests/posix2.tests`（3/27 失败：`$@` test、negative `test -x`、variable quoting 1）、
  `tests/posixexp.tests`、`tests/comsub-posix.tests`
- 现象：posix 模式下词切分/引号组合（"hello echo after 5" 应分行）、
  unterminated here-document 警告缺失。
- GNU 源码：`subst.c`（posix_mode 分支）、`eval.c`。

### G20 导出函数解析/导入
- 套件：`tests/exportfunc.tests`
- 现象：eval `X ( ) { (a)>\'` 解析错误位置与措辞；`unexpected EOF while looking for matching '}'`；
  `/dev/tty` 打开失败处理。
- GNU 源码：`variables.c`（导出函数导入/解析）、`parse.y`。

### G21 source bad interpreter 与 subshell 输出
- 套件：`tests/builtins.tests`
- 现象：`/var/tmp/x29-…: bash: bad interpreter` 错误缺失；`one.1 subshell`/`four.1 subshell`
  多打（source 与子 shell 执行路径差异）。
- GNU 源码：`execute_cmd.c:6184`（bad interpreter）；`builtins/source.def`。

### G22 函数内 `<(:)` 与特殊内建 break
- 套件：`tests/func.tests`
- 现象：line 45 缺 `<(:): command not found`；line 94 缺 `` `break': is a special builtin ``。
- GNU 源码：`parse.y`（procsub 解析）；`builtins/common.c`（special builtin 限制）。

### G23 控制字符经引号传递丢失
- 套件：`tests/nquote1.tests`、`tests/nquote4.tests`
- 现象：`v^A^A` → argv 得空串；`-en \01`；`ab^Lde` → `abde`（^L 丢失）。
- GNU 源码：`subst.c`（ansicstr）/`lib/sh/strtrans.c`（ANSI-C 引用展开）。

### G24 赋值 RHS 转义
- 套件：`tests/rhs-exp.tests`（105/105）
- 现象：`-DSELECT_VECS=\&m68kcoff_vec\>` 中 `\&`、`\>`、`\'` 的 RHS 保留错误。
- GNU 源码：`subst.c`（赋值词 RHS 展开路径）。

### G25 iquote DEL(^?) 引用
- 套件：`tests/iquote.tests`（92/95）
- 现象：`0x7f`/`^?` 系列输出缺失（非打印字符 quoting 规则不全）。
- GNU 源码：`subst.c` + `lib/sh/strtrans.c`。

### G26 new-exp 杂项
- 套件：`tests/new-exp.tests`（941/937）
- 现象：`${_ENV[(_$-=0)+(_=1)-_${-%%*i*}]}}` 错误展开、`ambiguous redirect` 消息、
  `$(< filename)` glob 失败用例、`HOME: }: arithmetic syntax error`（归 G11）。
- GNU 源码：`subst.c`。

### G27 POSIX 路径透传（已纠正归因，issue #99 已关闭）
- 原误判：winuxcmd cp 不接受 POSIX 绝对路径，需要 winuxcmd 侧修。
- 实测纠正：POSIX→Windows 参数转换是 shell 层职责且 rubash **已实现**
  （`src/executor/path.rs` `external_argument_path`：`/tmp`、`/dev/*`、`/mnt/X`
  无条件转换；`/bin`、`/etc`、`/usr`、`/var` 在配置 shell 根
  `WINUXSH_ROOT` 后经 `map_logical_path` 字面映射转换，niu.exe 实测正常）。
- 真正原因：gnu-compat harness 直接跑 rubash.exe 未配置 `WINUXSH_ROOT`，
  `/bin` 分支不激活，参数原样透传给 cp。
- 修复：harness 层 `prepare_posix_root` 构造最小 fixture 根
  （`bin/sh` 必须无 .exe 后缀字面名——`map_logical_path` 按字面名 exists()
  检查，MSYS cp 会把副本改名 `sh.exe`；另有 `etc/passwd` 与 tmp/var/home
  目录），`run_rubash` 导出 `WINUXSH_ROOT` 指向它。实测
  `cp /bin/sh /tmp/x` rc=0、`cat /etc/passwd` 正常。coreutils 侧无需改动。

## 分类 D：有意扩展（建议 check 白名单豁免，非缺陷）

| 套件 | rubash 专有输出 |
|------|----------------|
| shopt | `shopt -u completion_strip_exe`、`completion_strip_exe off`、`set +o igncr/restricted`、`igncr off` |
| comsub2 | `expand_aliases off` 行（待确认是否 POSIX 差异） |

## 处置路线

1. 分类 A → 文档化豁免 + 可修项已修（A4/A12 经 `prepare_posix_root` fixture 根解决，A2 转换层已通、消息差异豁免）。
2. 分类 B → G11/G12/G13/G14 批量修（纯消息层，风险低）。G11 上下文措辞已修（`3e348d53`）。
3. 分类 D → run-83.sh 归一化白名单（harness 层，已随 c22dfdb9 提交）。
4. 分类 C → 按本文档逐条开 issue，G18（挂起）与 G1/G2/G3（语义）优先。

---

## 2026-09-16 复核（feat/gnu-compat-batch1，HEAD 79aac755）

### 已修
- **G16 xtrace 算术/for/case/赋值前缀**（`565ba986` + `79aac755`）：
  ① 普通 `(( ))` 双打印去掉、保留括号间原文（GNU `+ ((  n  ))`）；
  ② for 头每迭代一行（`for i;` 形式打字面 `"$@"`）；
  ③ case 头一行（raw word）；
  ④ 赋值前缀独立成行（`foo=one echo hi` → `+ foo=one` `+ echo hi`）。
  上游 set-x.tests 剩余差异只剩两类子 gap（见下）。
- **实锤已修（ledger 过时项，双端探针一致）**：G8① read readonly stat=2、
  G13 trap -p/-P 校验、G9 $"..." locale、G10 词切分反斜杠、G23 控制字符、
  G1 set -e 花括号退出（`{ false; }` rc=1 一致）。

### set-x 剩余子 gap（本轮新定位）
- **BASH_XTRACEFD**：GNU 支持 trace fd 重定向（`exec 4>file` 后 xtrace 进
  文件），非法值报 `BASH_XTRACEFD: 4: invalid value for trace file
  descriptor`；rubash 完全未实现（print_cmd.c CHECK_XTRACEFD 语义）。
- **xtrace 词引用**：GNU 对 xtrace 词做 quoting（`metas=(\| \& ...)`、
  `n=($@)` 原样打印）；rubash 输出未引用形态（print_cmd.c
  xtrace_print_word_list 的 qtags 规则）。

### 新发现（下一轮候选）
- **varenv 非确定**：varenv9.sub readonly 局部/全局 scoping 两次运行结果
  不同（105↔107 行差异），master 同样存在，需定位。
- **混合行尾敏感**：set-x.tests 的 CRLF/LF 副本曾出现不同 xtrace 行为
  （复现不稳定，疑似 lexer/parser 状态泄漏，待隔离最小复现）。

### 当前 check 口径
PASS=34 / DIFF=48 / TIMEOUT=1（HEAD 79aac755，debug 构建，150s 超时）。
大头：intl 1378（i18n 长期项）、errors 385（G11/G12 消息层）、array 242、
assoc 200、nameref 196（G5-G7 数组族）。
