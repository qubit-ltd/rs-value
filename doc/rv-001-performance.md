# RV-001 基线采样记录

本记录位于 rs-value `doc/rv-001-performance.md`，对应 rs-config worktree `codex/rv001-config-wire`。记录 T1 旧路径基线与 T2 prepared-view 候选的对比；分配成本只记录源码层面的额外缓存，不包含可靠的分配器计数。

## 输入与配置

| Criterion ID | 固定输入 | 成功编码字节数 |
| --- | --- | ---: |
| `config_wire_encode/small_config_4_i64_properties` | 4 个 i64 属性 | 535 |
| `config_wire_encode/one_json_100000_integer_nodes` | 单个含 100,000 个整数节点的 JSON 数组 | 589,054 |
| `config_wire_encode/100_json_properties_1000_integer_nodes_each` | 100 个属性，各含 1,000 个整数节点的 JSON 数组 | 401,547 |
| `config_wire_encode/one_f64_collection_100000_values` | 单个含 100,000 个有限 f64 值的属性 | 689,069 |

所有输入均在 Criterion 计时区外构造。每个 fixture 在注册计时前用同一 `ConfigWireLimits` 成功编码一次。配置继承默认 JSON 限制，仅将最大输出字节和累计 payload 字节提高到 4 MiB、最大节点数提高到 200,000、单数组最大元素数提高到 100,000，以容纳计划中的固定输入。计时操作是 `Config::encode_json_vec_with_limits`，返回字节由 `black_box` 消费。

## 环境

- 采样时间：2026-11-08 UTC。
- 主机：Linux `mas-server`，内核 `6.8.0-147-generic`，x86_64。
- CPU：2 × Intel Xeon E5-2680 v4 @ 2.40 GHz；共 28 个物理核、56 个逻辑 CPU。采样时 CPU scaling 状态由系统报告为 49%。
- 内存：125 GiB；文件系统为 `/dev/md0p1`，可用空间约 3.3 TiB。
- Rust：`rustc 1.94.0 (4a4ef493e 2026-03-02)`，`cargo 1.94.0 (85eff7c80 2026-01-15)`；项目 `rust-toolchain.toml` 固定 `1.94.0`。
- Criterion：`0.8.2`；构建 profile：`bench`（optimized）。未传 feature 参数，qubit-config 默认 feature 集合为空；bench 使用 dev-dependency `serde_json/arbitrary_precision` 和 `criterion`（`default-features = false`）。
- 本地源码依赖：Cargo 解析 `qubit-value 0.13.0` 到当前 workspace 的 rs-value checkout（worktree 中的 `../rs-value` 链接），不是 registry 包。
- Cargo target：未设置 `CARGO_TARGET_DIR`、未发现 `.cargo/config.toml` 重定向；使用本 worktree 的 `target/`，避免复用其他 worktree 的 build artifacts。Cargo registry cache 是机器共享的。
- 采样期间主机上有多个其他仓库的 Cargo test/build 进程并发运行。因此 CPU/IO 干扰无法排除，跨次波动较大；这些结果不适合单独支撑候选的净收益结论。

## 三次独立运行

每次均在 rs-config worktree 根目录分别启动新进程，命令完全相同。Criterion 的三点时间区间按其默认 95% 置信水平输出；以下保留每次的估计点和区间，时间单位如表头。

```text
cargo bench --bench config_wire_encode_bench -- --save-baseline rv001-before
```

| 运行 | 命令退出码 | 小配置 (µs) | 单大 JSON (ms) | 100 个 JSON 属性 (ms) | 100k f64 (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 0 | 4.6855 [4.5713, 4.8353] | 94.303 [90.359, 98.802] | 42.608 [41.529, 43.941] | 15.152 [14.964, 15.381] |
| 2 | 0 | 4.4927 [4.4818, 4.5078] | 45.189 [44.223, 46.344] | 65.310 [62.420, 68.092] | 16.406 [15.910, 16.958] |
| 3 | 0 | 4.6580 [4.6485, 4.6678] | 83.712 [81.368, 86.438] | 41.157 [40.967, 41.463] | 15.443 [15.166, 15.762] |

Criterion 每次报告的 100 个样本离群比例：运行 1 各组为 11%、5%、15%、10%；运行 2 为 13%、15%、19%、19%；运行 3 为 5%、12%、9%、11%。运行 2、3 的 `change` 是相对前一次写入的同名 `rv001-before` 基线计算；由于指定命令每次都复用该标签，变化百分比不代表任何优化比较。

## 数字摘要与限制

三次每项 Criterion 估计点的跨次中位数（括号为三次估计点的最小至最大值）为：小配置 4.658 µs（4.493–4.686）；单大 JSON 83.712 ms（45.189–94.303）；100 个 JSON 属性 42.608 ms（41.157–65.310）；100k f64 15.443 ms（15.152–16.406）。大 JSON 两组跨进程波动明显，且有 9–19% 的样本被 Criterion 标记为离群值，因此保留各次结果而不把单次估计当作确定性能值。

## T2 候选对比

候选为 bounded encoder 缓存每个成功预检后的 `ValueWireRefV1`，用内部 prepared DTO 序列化，而普通 Serde 路径不变。三次均单独启动新进程并使用：

```text
cargo bench --bench config_wire_encode_bench -- --baseline rv001-before
```

每次命令退出码为 0。为了固定比较基线，在采样前将 T1 的 `target/criterion/config_wire_encode_*/rv001-before` 文件复制到 worktree `target/rv001-before-snapshot/` 并计算 SHA-256；候选运行前后快照与 Criterion 中 `rv001-before` 目录逐文件相同。基线估计点与 T1 第三次记录相符。Criterion `change` 列是新候选相对该保存基线的变化，负数表示更快；区间为 Criterion 默认的 95% 置信区间。

| 运行 | 命令退出码 | 小配置：时间；变化 | 单大 JSON：时间；变化 | 100 个 JSON 属性：时间；变化 | 100k f64：时间；变化 |
| --- | ---: | --- | --- | --- | --- |
| 1 | 0 | 4.5147 µs；−3.11% [−3.86%, −2.32%] | 37.693 ms；−54.97% [−56.40%, −53.64%] | 36.367 ms；−11.64% [−12.62%, −10.73%] | 15.081 ms；−2.34% [−4.51%, −0.23%]，Criterion 判为噪声内 |
| 2 | 0 | 4.9869 µs；−0.07% [−2.86%, +3.03%]，无显著变化 | 38.752 ms；−53.71% [−55.26%, −52.17%] | 38.382 ms；−6.74% [−10.40%, −1.89%] | 15.701 ms；+1.67% [−2.34%, +6.14%]，无显著变化 |
| 3 | 0 | 4.5653 µs；−2.48% [−3.22%, −1.71%] | 37.756 ms；−54.90% [−56.35%, −53.58%] | 36.061 ms；−12.38% [−13.07%, −11.92%] | 17.330 ms；+12.22% [+6.63%, +18.38%] |

每次各组 100 个样本的离群比例：运行 1 为 5%、8%、11%、10%；运行 2 为 11%、9%、14%、16%；运行 3 为 13%、5%、6%、14%。其中离群类型详见各次 Criterion 输出；运行 3 的 f64 样本被报告为 14 个高轻度离群值。

三次候选估计点的跨次中位数（范围）为：小配置 4.565 µs（4.515–4.987）；单大 JSON 37.756 ms（37.693–38.752）；100 个 JSON 属性 36.367 ms（36.061–38.382）；100k f64 15.701 ms（15.081–17.330）。对应中位变化约为 −2.48%、−54.90%、−11.64%、+1.67%。大 JSON 单属性与多属性两组均在三次候选采样中稳定变快；小配置没有稳定超过 5% 的回退。100k f64 没有稳定改善：两次与基线无显著差异，第三次显著回退 12.22%。

候选运行时 CPU 空闲率在运行前观察约 92.8–92.9%，但机器仍有其他仓库的 Cargo 工作；1 分钟 load average 约 13.9–15.2，5 分钟约 24.1–26.4。T1 与候选不是同一负载时段的配对采样，且 T1 大输入基线本身跨次波动大，因此百分比是对固定 T1 Criterion artifact 的比较，不消除跨时段主机负载差异。

## 分配成本与阶段性判断

候选 `Vec::with_capacity(property_count)` 为非空配置增加一个源码层面的缓存分配，容量随属性数为 O(property_count)；各项属性和值仍是借用引用，没有逐属性堆分配或值克隆。此次没有可靠的分配器计数，未测出实际分配次数、请求字节或峰值内存，因此分配成本尚未量化。

当前测量表明 JSON 主场景显著改善，且小配置未出现超过 5% 的稳定回退；但计划要求大浮点集合也应稳定改善并超过噪声，本次 f64 结果不满足该条件，且额外缓存分配未量化。候选的完整净收益证据不足，因此 T3 撤去生产候选，保留基准和兼容回归测试；本轮结论为“已核查，暂不实施优化”。

## T3 最终验证与裁决

- 撤去了 `Prepared*WireRef` 和 bounded encoder 改动；保留四类输入基准与普通 Serde 字节一致、round-trip、错误优先级、非有限浮点及 arbitrary-precision JSON number 回归测试。
- 候选与固定 T1 基线相比，大 JSON 单属性和多属性中位变化分别为 −54.90% 和 −11.64%；小配置为 −2.48%；100k f64 为 +1.67%（两轮处于噪声内，一轮显著回退 12.22%）。按设计中的大浮点场景稳定收益要求，候选未达标。未测量 allocator 请求字节/次数，故不作“净性能已证明”的表述。
- 格式化对齐后，`cargo test --test config_wire_tests` 退出码 0（63 passed），`cargo test --all-features` 退出码 0，Criterion 基准 smoke 命令 `cargo bench --bench config_wire_encode_bench -- --warm-up-time 1 --measurement-time 1 --sample-size 10` 退出码 0。
- `.infra/bin/ci-check.sh` 首次退出码 1，报告项目 rustfmt 格式差异；执行 `.infra/bin/align-ci.sh` 对齐后重新运行 `.infra/bin/ci-check.sh` 退出码 0。CI 的 coverage 阶段一并成功：functions 99.46%（915/920）、lines 97.33%（6276/6448）、regions 96.17%（9496/9874），均超过仓库门槛。因此没有再单独重复 coverage 脚本。
