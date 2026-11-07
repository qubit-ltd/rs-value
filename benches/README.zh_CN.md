# 基准测试

`downstream_read_bench` 测量下游 crate 在 Wire V1 中检查并编码值时使用的读取路径。
`preflight/`、`borrowed_encode/` 和 `preflight_and_encode/` 三组分别覆盖 1、32 和
4096 个字符串值。测试夹具使用足够大的预算，使 4096 项输出能够成功编码，避免把
预算拒绝误当成性能数据。

三组分别计时以下步骤：

| 组名 | 计时范围 |
| --- | --- |
| `preflight/` | 仅显式 payload preflight。 |
| `borrowed_encode/` | DTO 在计时开始前构造；计时内由 helper 执行内部 preflight 并编码借用的 DTO。 |
| `preflight_and_encode/` | 显式 preflight、借用 DTO 构造、helper 内部 preflight 和编码都在计时内。 |

`preflight_and_encode/` 与 `borrowed_encode/` 的差值还包含借用 DTO 的构造成本，不能解释为纯
preflight 成本。

运行定向基准测试：

```text
cargo bench --all-features --bench downstream_read_bench
```

Criterion 输出是本次变更的性能证据。在当前环境无法运行基准测试时，必须明确报告
不可用，不能据此宣称性能改进。

## 重复构造借用 DTO 校验（2026-10-30）

本次定向对照运行于 Intel Core i5-9600K（6 核）、Ubuntu 24.04 x86_64，Rust/Cargo
1.94.0，使用优化后的 Criterion bench profile。三轮均使用同一命令：

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- repeated_validation --sample-size 50 --warm-up-time 3 --measurement-time 5
```

每个 fixture 含 4096 个值。编码输出长度分别为：字符串 28,721 字节、有限浮点数
23,526 字节、含嵌套数字的 JSON 对象 133,005 字节。`single` 计时显式 preflight、一次借用
DTO 构造和有界编码；`repeated` 在同一 preflight 与最终 DTO 构造之间额外构造并丢弃一个借用
DTO。计时前断言两条路径输出字节一致。fixture 在计时区外构造，两条路径都使用 4 MiB 输出限额。

三轮 Criterion 区间（下限–上限；低于 1 ms 的值以微秒表示）：

| Fixture | 轮次 | 单次 | 重复 |
| --- | ---: | ---: | ---: |
| strings | 1 | 1.5054–1.9183 ms | 699.94 µs–1.1108 ms |
| strings | 2 | 567.78–793.42 µs | 556.51–846.03 µs |
| strings | 3 | 645.50–854.52 µs | 936.18 µs–1.2867 ms |
| floats | 1 | 1.8496–2.4945 ms | 2.4075–3.2211 ms |
| floats | 2 | 910.16 µs–1.2461 ms | 1.6974–2.3325 ms |
| floats | 3 | 1.3558–1.7379 ms | 1.2526–1.8670 ms |
| json | 1 | 30.692–41.681 ms | 15.335–21.009 ms |
| json | 2 | 39.920–54.924 ms | 26.554–37.858 ms |
| json | 3 | 19.641–25.933 ms | 15.910–19.552 ms |

比较应使用每轮的 single/repeated 区间，而不是 Criterion 相对上次保存基线的 `change` 估算。同轮中位数
比值（重复相对单次）为：strings −48.0%、+2.8%、+47.5%；floats +28.5%、+83.8%、+0.9%；JSON
−49.7%、−31.3%、−22.1%。只有第二轮 floats 的区间不相交且中位数额外耗时超过 5%；第一轮 floats
区间略有重叠，第三轮接近持平。JSON 第三轮区间虽不相交，但重复构造反而更快。因此，支持“额外耗时超过
5%”且区间分离的轮次少于两轮，没有达到设计规定的真实路径后续核查筛选条件。轮次间绝对耗时也有明显漂移。
这些结果不支持生产代码修改；本代理基准未证实重复校验值得优化。

该基准隔离的是额外一次值表示校验成本，并非 `rs-config` 端到端基准：它不包含配置遍历、属性
序列化、外围 JSON 编码 session 与预算计费。测量主机保留的最终 benchmark 源码 Criterion 原始输出位于
`/tmp/rv-003-final-bench-1.txt`、`/tmp/rv-003-final-bench-2.txt` 和 `/tmp/rv-003-final-bench-3.txt`。

## JSON 数字 DTO 验证（2026-11-04）

本组测量借用 payload DTO 对普通浮点 JSON 数组的验证，以及相同 DTO 构造加有界编码的耗时。
fixture 分别含 256 或 4096 个值（`index as f64 / 10.0`），在计时区外构造，使用 4 MiB 输出
限额，并在测量前断言可以成功编码。配置使用标准 `serde_json` 数字存储（`natural-json`，未启用
`arbitrary_precision`）。

优化前后每轮均使用相同命令：

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- nested_json_number_validation --sample-size 50 --warm-up-time 3 --measurement-time 5
```

测量环境为 Intel Xeon E5-2680 v4、Ubuntu 24.04 x86_64、Rust/Cargo 1.94.0。下表为 Criterion
下限–上限；低于 1 ms 的耗时以微秒表示。

| Fixture | 轮次 | DTO 构造 | DTO 构造 + 有界编码 |
| --- | ---: | ---: | ---: |
| 256 | 基线 1 | 25.517–31.761 µs | 84.463–100.97 µs |
| 256 | 基线 2 | 22.742–22.868 µs | 74.507–77.384 µs |
| 256 | 基线 3 | 23.242–23.814 µs | 73.632–73.856 µs |
| 4096 | 基线 1 | 477.17–619.85 µs | 1.2429–1.3656 ms |
| 4096 | 基线 2 | 459.10–583.86 µs | 1.5088–1.8669 ms |
| 4096 | 基线 3 | 379.31–394.16 µs | 1.2042–1.3087 ms |

| Fixture | 轮次 | DTO 构造 | DTO 构造 + 有界编码 |
| --- | ---: | ---: | ---: |
| 256 | 优化后 1 | 21.737–21.931 µs | 97.947–129.18 µs |
| 256 | 优化后 2 | 21.787–21.926 µs | 71.713–71.995 µs |
| 256 | 优化后 3 | 22.312–27.521 µs | 74.369–79.914 µs |
| 4096 | 优化后 1 | 385.14–442.47 µs | 1.2025–1.2493 ms |
| 4096 | 优化后 2 | 370.02–455.47 µs | 1.1559–1.1571 ms |
| 4096 | 优化后 3 | 374.87–402.23 µs | 1.1684–1.1777 ms |

DTO 构造耗时区间重叠或有所改善。构造加编码耗时轮次间波动，但没有两轮同时出现超过 5% 且区间
不重叠的退化。按设计中的决策门槛保留栈缓冲修改；计时数据噪声较大，不能据此确认下游端到端提速。

线程局部分配测试在 fixture 构造和预热后测量 payload DTO 构造。修改前，整数数组在三个尺寸下均分配
288 B；浮点数组在尺寸 1、256、4096 时分别分配 296 B、2,336 B 和 33,056 B。修改后，整数和浮点
fixture 在三个尺寸下均分配 288 B。该数据包含遍历帧的固定分配，说明普通数字不再逐个临时分配字符串。
`arbitrary_precision` 行为另行验证；其数字表示可能仍有内部堆分配，长数字文本仍使用有分配的回退路径。
这不表示整个 Wire V1 编码过程都无分配。

## 规范 JSON 对象（RV-002）

`canonical_object/{flat,nested}/{ordered,reverse,shuffled}/{0,1,16,1024}/{wire,natural}` 测量不同键值对象的 Wire V1 JSON 序列化和自然 JSON 物化。确定性输入和 Wire DTO 均在计时前构建。`shuffled` 使用固定置换；默认 `serde_json` map 后端会规范化插入顺序，而启用 `preserve_order` 时会覆盖非规范遍历。`nested` 将每个值置于两层对象中。

默认配置与统一启用 `preserve_order`、`arbitrary_precision` 的配置分别保存并比较：

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- canonical_object --sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline rv002-default-before
cargo bench --locked --no-default-features --features 'natural-json,serde_json/preserve_order,serde_json/arbitrary_precision' --bench downstream_read_bench -- canonical_object --sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline rv002-unified-before
# 实现后分别将 --save-baseline 改为 --baseline，并使用相应基线名。
```

保持工具链、机器、profile、输入和 Criterion 参数（`--sample-size 10 --warm-up-time 1 --measurement-time 1`）一致。Criterion 有噪声且这里只覆盖合成 JSON 树操作；孤立的小幅变化应视为不确定，稳定超过 5% 的回退需要重复调查。分配证据应与吞吐量分开报告。

## RV-002 结果与限制（2026-11-06）

前后测量使用 Rust 1.94.0、Criterion 优化 bench profile，以及 Intel Xeon E5-2680 v4（56 个逻辑 CPU，Ubuntu 24.04 x86_64）。部分样本期间同一主机还运行了其他 Cargo 构建，因此短测存在较大离群值和跨轮波动。两种配置均采用 10 个样本、1 秒预热和 1 秒测量。对疑似 default 配置 flat/ordered/1024 Wire 回退，另以 30 个样本、3 秒预热和 5 秒测量复跑。

| 配置与场景 | 修改前区间 | 修改后区间 | 解释 |
| --- | ---: | ---: | --- |
| Default，flat ordered 1024 Wire（首次） | 97.6–101.7 µs | 102.8–111.2 µs | 表面回退约 6.5%；延长复跑为 100.7–105.8 µs，Criterion change 区间 −1.5% 至 +4.9%、p=0.40，未确认回退。 |
| 统一 features，flat ordered 1024 Wire | 92.2–94.9 µs | 88.4–89.1 µs | 短测约快 5%；结果有参考意义，但不能外推为端到端性能结论。 |
| 统一 features，nested ordered 1024 Natural JSON（临时实验，已撤回） | 1.25–1.35 ms | 1.25–1.42 ms | 未确认变化（p=0.98）。 |
| 统一 features，flat shuffled 1024 Natural JSON（临时实验，已撤回） | 434–455 µs | 447–513 µs | 无序对象仍显示超过 5% 的回退信号（Criterion change 区间 +20% 至 +44%、p<0.001）。 |

分配观测隔离了 1024 键 Wire 对象序列化，fixture、DTO 构造和 canonical 输出校验均在计时外。测量使用 `serde_json::to_writer(sink(), &wire)`，不包含输出缓冲分配。默认 `serde_json` 下两种 map 遍历都已排序，sink 分配均为 0 bytes；统一 `preserve_order` + `arbitrary_precision` 下，有序输入为 0 bytes，逆序输入为 16,384 bytes，正好是排序索引向量的分配。两种输入在计时外都验证为相同的 16,319-byte canonical 输出。此前用 `to_json_vec()` 得到的分配总数包含返回的输出 Vec，不能作为此隔离测量的对比数据，已从此处移除。

保留 Wire 快速路径：它确实移除了已排序对象的排序索引分配，原预算拒绝测试仍通过，也未发现确认的吞吐回退。Natural JSON 快速路径曾作为 `canonicalize_json_value` 的本地临时实验；上表数据测量了该原型。由于无序对象复测仍有回退，该路径在最终提交前已撤回，当前提交代码不含此路径。这些都是合成对象测量，不能代表完整下游应用性能。原始 Criterion 报告和采样数据保存在测量主机的 `target/criterion` 下。

## Natural JSON 宽数组分配证据

独立的 `natural_json_allocation_tests` integration binary 测量公开 `Value::to_json_value` 路径：fixture 在测量窗口外构造，投影结果保留到读数完成。线程局部全局分配器记录累计请求分配字节（成功 `alloc` 的大小加成功 `realloc` 的新大小）和峰值存活请求字节（alloc/realloc 增量减去 dealloc/realloc 旧大小）。直接构造 `serde_json::Value::Array(vec![Bool(true); n])` 作为必需输出参照。这些是测试 profile 中分配器请求的字节，不是 RSS 或 Criterion 吞吐量。

测量环境为 Linux x86_64、rustc 1.94.0；默认 profile 下 `serde_json::Value` 大小为 32 字节，启用 `arbitrary_precision,preserve_order` 后为 72 字节。fixture 是由 `n` 个布尔值组成的数组，测量 16、1024 和 4096 项。默认 profile 命令：`cargo test --locked --features natural-json --test natural_json_allocation_tests -- --nocapture`。unified profile 命令：`cargo test --locked --features 'natural-json,serde_json/arbitrary_precision,serde_json/preserve_order' --test natural_json_allocation_tests -- --nocapture`。

| Feature profile | 项数 | 投影累计 / 峰值存活（B） | 直接输出累计 / 峰值存活（B） | 投影超出输出的临时量（B） |
| --- | ---: | ---: | ---: | ---: |
| default，修改前 | 16 | 2,016 / 1,296 | 512 / 512 | 1,504 / 784 |
| default，修改前 | 1,024 | 114,912 / 81,936 | 32,768 / 32,768 | 82,144 / 49,168 |
| default，修改前 | 4,096 | 458,976 / 327,696 | 131,072 / 131,072 | 327,904 / 196,624 |
| default，修改后 | 16 | 1,280 / 960 | 512 / 512 | 768 / 448 |
| default，修改后 | 1,024 | 33,536 / 33,216 | 32,768 / 32,768 | 768 / 448 |
| default，修改后 | 4,096 | 131,840 / 131,520 | 131,072 / 131,072 | 768 / 448 |
| unified，修改前 | 16 | 3,584 / 2,576 | 1,152 / 1,152 | 2,432 / 1,424 |
| unified，修改前 | 1,024 | 237,440 / 163,856 | 73,728 / 73,728 | 163,712 / 90,128 |
| unified，修改前 | 4,096 | 950,144 / 655,376 | 294,912 / 294,912 | 655,232 / 360,464 |
| unified，修改后 | 16 | 1,504 / 1,376 | 1,152 / 1,152 | 352 / 224 |
| unified，修改后 | 1,024 | 74,080 / 73,952 | 73,728 / 73,728 | 352 / 224 |
| unified，修改后 | 4,096 | 295,264 / 295,136 | 294,912 / 294,912 | 352 / 224 |

4096 项时，default profile 的累计 / 峰值存活字节分别减少 327,136 / 196,176 B，unified profile 分别减少 654,880 / 360,240 B。修改后的样本中，直接输出参照以外的临时分配不随宽度增长。回归测试要求 4096 项临时量不超过 16 项临时量加两个 `Value` 槽位；未修改的实现会违反此边界。这些数据只证明该合成布尔数组投影的分配器流量，不代表进程 RSS 或一般 JSON 负载。

数组 Criterion 对比使用同一基线源码修订版（`6f5ae6e`）和同一 benchmark fixture：修改前在独立 detached worktree 中测量，修改后在实施 worktree 中测量。命令使用 `--sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline`；feature 分别为 `natural-json` 和 `natural-json,serde_json/arbitrary_precision,serde_json/preserve_order`。fixture 在计时区外创建，投影结果通过 `black_box` 返回。下表为 Criterion 95% 区间：

| Profile | 项数 | 修改前 | 修改后 | 中位数变化 |
| --- | ---: | ---: | ---: | ---: |
| Default | 16 | 884.33–887.40 ns | 729.38–734.05 ns | −17.3% |
| Default | 1,024 | 37.103–37.520 µs | 34.546–34.631 µs | −7.2% |
| Default | 4,096 | 146.58–146.68 µs | 137.90–137.96 µs | −6.0% |
| Unified | 16 | 1.0146–1.0178 µs | 797.15–797.92 ns | −21.5% |
| Unified | 1,024 | 47.128–47.212 µs | 41.559–41.594 µs | −11.8% |
| Unified | 4,096 | 190.95–191.32 µs | 170.19–172.06 µs | −10.6% |

对象对比重新测量了与规范排序最相关的 Natural JSON 投影用例。修改前后均使用 10 个样本、1 秒预热和 1 秒测量；对象 fixture 在计时区外创建。下表报告 Criterion 95% slope 区间：

| Profile | 1,024 项对象用例 | 修改前 | 修改后 |
| --- | --- | ---: | ---: |
| Default | 平坦、有序 | 376.3–389.4 µs | 378.9–379.8 µs |
| Default | 嵌套、有序 | 1.311–1.364 ms | 1.272–1.273 ms |
| Default | 平坦、打乱顺序 | 414.5–419.6 µs | 373.1–373.4 µs |
| Unified | 平坦、有序 | 309.9–352.1 µs | 294.5–295.2 µs |
| Unified | 嵌套、有序 | 1.363–1.434 ms | 1.208–1.224 ms |
| Unified | 平坦、打乱顺序 | 431.9–450.5 µs | 403.9–405.3 µs |

这些合成测试没有显示可重复的对象性能回退；default 平坦有序区间重叠，其余所列修改后区间更低。本轮数组计时也有所改善，但两者都不能代表进程 RSS 或端到端应用性能。Criterion 结果会受测量主机及其负载影响。

## JSON 根叶节点身份哈希（RV-001）

根叶节点优化让 JSON null、布尔值、数值和字符串绕过迭代遍历 Vec；数组和对象仍使用原迭代引擎。分配计数测试在计量区间外构造夹具和 hasher，并预热哈希路径及线程局部分配器，只测 `Value::hash`。测试将 JSON 整数哈希与直接 `serde_json::Number::hash` 对照，以区分遍历状态分配和 Number 自身的哈希成本；另用较大词法数值对照两条路径，在 unified 配置下覆盖 `arbitrary_precision` Number。

优化前分配基线使用
`cargo test --locked --features json --test json_identity_allocation_tests -- --nocapture`
和
`cargo test --locked --features 'all,serde_json/arbitrary_precision,serde_json/preserve_order' --test json_identity_allocation_tests -- --nocapture`
测量。原实现会按预期因新的零分配断言失败，但会打印测量值；对应的优化后命令通过。

| 配置 | 哈希值 | 优化前：次数 / 字节 | 优化后：次数 / 字节 | 直接 Number：次数 / 字节 |
| --- | --- | ---: | ---: | ---: |
| 默认 | null、bool、integer、float、空/非空字符串（每种） | 2 / 160 | 0 / 0 | integer 0 / 0 |
| unified `arbitrary_precision` + `preserve_order` | null、bool、integer、float、空/非空字符串（每种） | 2 / 120 | 0 / 0 | integer 0 / 0 |

较大词法数值 `12345678901234567890.123456789` 在 unified 优化前通过 `Value::Json` 哈希分配 2 次 / 120 字节，直接 Number 哈希为 0 / 0；优化后两者均为 0 / 0。测试断言两条路径的分配量相等，不承诺所有 Number 配置都零分配。

优化前后的 Criterion 测量均使用 Rust/Cargo 1.94.0、Linux x86_64、Intel Xeon E5-2680 v4、`--all-features`；每个样例 30 个采样、预热 1 秒、测量 2 秒。命令为：

```text
cargo bench --locked --all-features --bench value_identity_bench -- 'identity/hash_json_' --sample-size 30 --warm-up-time 1 --measurement-time 2
```

每格为 Criterion 中位估计值及 95% 区间。以下是优化前后三轮完整结果。区间波动很大，尤其是容器样例，因此计时证据不足以确认稳定加速或回退；可重复结果是遍历分配减少。

| 样例 | 优化前第 1 轮 | 优化前第 2 轮 | 优化前第 3 轮 |
| --- | ---: | ---: | ---: |
| null | 58.182（56.909–60.607 ns） | 56.947（56.781–57.204 ns） | 57.009（56.551–57.864 ns） |
| bool | 67.126（64.025–71.544 ns） | 63.523（61.534–66.862 ns） | 61.193（61.068–61.326 ns） |
| integer | 67.340（67.080–67.765 ns） | 67.037（66.803–67.308 ns） | 68.572（66.988–70.955 ns） |
| float | 67.365（66.895–68.054 ns） | 68.941（67.731–70.253 ns） | 66.854（66.737–67.028 ns） |
| 空字符串 | 64.893（64.674–65.142 ns） | 65.723（65.305–66.447 ns） | 64.598（64.577–64.622 ns） |
| 非空字符串 | 72.538（71.037–74.532 ns） | 73.784（71.249–79.023 ns） | 71.456（70.711–72.525 ns） |
| 浅层数组 | 250.00（249.61–250.59 ns） | 552.88（496.13–628.74 ns） | 257.21（256.68–257.91 ns） |
| 嵌套 JSON | 1.4277（1.4122–1.4510 µs） | 1.9655（1.6847–2.3480 µs） | 1.4172（1.4087–1.4279 µs） |

| 样例 | 优化后第 1 轮 | 优化后第 2 轮 | 优化后第 3 轮 |
| --- | ---: | ---: | ---: |
| null | 21.869（21.435–22.547 ns） | 32.846（28.383–37.580 ns） | 31.174（27.720–35.723 ns） |
| bool | 25.805（25.705–25.946 ns） | 26.584（26.527–26.644 ns） | 26.661（25.900–27.511 ns） |
| integer | 52.518（45.575–57.627 ns） | 61.728（58.521–64.699 ns） | 32.866（31.586–34.936 ns） |
| float | 122.70（97.936–147.23 ns） | 56.865（55.713–58.594 ns） | 32.458（32.015–33.241 ns） |
| 空字符串 | 63.352（57.731–69.965 ns） | 33.591（31.852–36.194 ns） | 34.399（30.902–38.325 ns） |
| 非空字符串 | 73.966（67.075–80.775 ns） | 46.505（41.263–51.240 ns） | 44.252（40.016–49.302 ns） |
| 浅层数组 | 581.88（496.70–675.79 ns） | 257.66（256.72–259.22 ns） | 514.28（449.95–575.22 ns） |
| 嵌套 JSON | 3.8822（3.3566–4.5037 µs） | 1.4277（1.4075–1.4654 µs） | 2.8551（2.6866–3.0510 µs） |

测量时主机调度波动明显。例如浅层数组的中位数优化前为 250–553 ns，优化后为 258–582 ns；嵌套 JSON 优化前为 1.42–1.97 µs，优化后为 1.43–3.88 µs。因此保留快速路径的依据是根叶节点确实少了两次遍历分配，不是吞吐提升结论。这些合成根节点样例也不能代表应用端到端性能。
