# glytex - a GPU based miner for Tari

Originally based on @cookiemonstermayhem's xtrgpuminer

## Quick Usage Instructions
### Prerequisites
The following are prerequisites for running solo mining Tari via the GPU using `glytex`:
- An active MinoTari node (require gRPC address from node)
- A MinoTari Wallet Address
- (Optional): Running `minotari_console_wallet` to view and review block rewards.

### Instructions
Download the latest release here: https://github.com/tari-project/glytex/releases

Be sure to select the version that is appropriate for the network you want to mine for. In most cases, you'll want `mainnet`

Unzip, then run the following command: 

- If you're running the miner on the same local machine where the MinoTari node is: `./glytex --tari-address <address> --engine OpenCL --http-server-port <desired_glytex_http_port>`. This should automatically locate the MinoTari node if using default settings.
- If the MinoTari node has a different address or is located remotely: `./glytex --tari-address <address> --engine OpenCL --tari-node-url <ipaddress>:<port> --http-server-port <desired_glytex_http_port>`

These should start the miner. By default, `glytex` uses all compatible GPUs and automatically optimizes mining parameters for about one second per cycle, with minimal CPU and memory usage.

**Sample Output**
```sh
./glytex --tari-address f2BcXXm8wrezYAvqH4KcXAEKP6srsiGLjjBVAYyUAEiBFFPU9BKYzqSaupz44ZA9vTiYZbAebjhKMernkzhj6kvS1nF --engine OpenCL --http-server-port 9696
Device: Apple M1 is available: true is excluded false
Device indexes to use: 1 from the total number of devices: 1
Device index: 0
Starting thread for device index: 0
Connecting to http://127.0.0.1:18142
Connecting to http://127.0.0.1:18142
Starting HTTP server at http://127.0.0.1:9696
Starting HTTP listener address Ok(127.0.0.1:9696)
Current height: 69659, P2Pool height: 0 time: 1s
Refreshing block template
Getting block template
Block template refreshed
Current height: 69659, P2Pool height: 0 time: 1s
Current height: 69659, P2Pool height: 0 time: 2s
[Thread:0] total 55,050,240 grid: 1024 max_diff: 14,852,555, target: 820,914,370 hashes/sec: 27,525,120
Current height: 69659, P2Pool height: 0 time: 3s
Current height: 69659, P2Pool height: 0 time: 4s
[Thread:0] total 110,100,480 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,525,120
Current height: 69659, P2Pool height: 0 time: 5s
Current height: 69659, P2Pool height: 0 time: 6s
[Thread:0] total 165,150,720 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,525,120
Current height: 69659, P2Pool height: 0 time: 7s
Current height: 69659, P2Pool height: 0 time: 8s
[Thread:0] total 221,118,464 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,639,808
Current height: 69659, P2Pool height: 0 time: 9s
Current height: 69659, P2Pool height: 0 time: 10s
[Thread:0] total 277,086,208 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,708,620
Current height: 69659, P2Pool height: 0 time: 11s
Current height: 69659, P2Pool height: 0 time: 12s
[Thread:0] total 332,136,448 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,678,037
Current height: 69659, P2Pool height: 0 time: 13s
Current height: 69659, P2Pool height: 0 time: 14s
[Thread:0] total 387,186,688 grid: 1024 max_diff: 23,439,791, target: 820,914,370 hashes/sec: 27,656,192
Current height: 69659, P2Pool height: 0 time: 15s
Current height: 69659, P2Pool height: 0 time: 16s
output and diff 424168480 70623
Block submitted
Current height: 69660, P2Pool height: 0 time: 17s
Refreshing block template
Getting block template
Block template refreshed
Current height: 69660, P2Pool height: 0 time: 1s
```

Congratulations, you're now mining with `glytex`. You can use the `minotari_console_wallet` to confirm your rewards.

## How to Customise Resource Utilization

You can customize resource utilization with additional command line options or configuration file entries. Key options:

**Command-Line Flags**

    --block-size <value>
    Sets the number of threads per block for GPU mining.
    --grid-size <value>
    Sets the grid size for GPU mining (number of blocks).
    --iterations-per-cycle <value>
    Controls how many iterations the miner does in each mining cycle.
    --engine <OpenCL|Metal>
    Select GPU engine (OpenCL for most GPUs, Metal for Apple devices).
    --gpu-status-file <path>
    Specify a custom GPU status file to control which GPUs are used.
    --detect
    Forces device detection and status file creation.
    --find-optimal
    Runs a benchmark to find the optimal grid size for your device.
    --benchmark
    Runs a performance benchmark instead of mining.