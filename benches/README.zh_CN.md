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
