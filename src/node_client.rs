use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use log::{error, info, warn};
use minotari_app_grpc::tari_rpc::{
    base_node_client::BaseNodeClient,
    pow_algo::PowAlgos,
    Block,
    Empty,
    GetNewBlockResult,
    NewBlockTemplate,
    NewBlockTemplateRequest,
    NewBlockTemplateResponse,
    PowAlgo,
};
use serde_json::json;
use tari_common::MAX_GRPC_MESSAGE_SIZE;
use tari_common_types::{tari_address::TariAddress, types::FixedHash};
use tonic::{async_trait, transport::Channel};

use crate::{p2pool_client::P2poolClientWrapper, ConfigFile};

const LOG_TARGET: &str = "tari::gpuminer::node-client";

pub(crate) struct BaseNodeClientWrapper {
    client: BaseNodeClient<tonic::transport::Channel>,
}

impl BaseNodeClientWrapper {
    pub async fn connect(url: &str) -> Result<Self, anyhow::Error> {
        println!("Connecting to {}", url);
        info!(target: LOG_TARGET, "Connecting to {}", url);
        let mut client: Option<BaseNodeClient<Channel>> = None;
        while client.is_none() {
            match BaseNodeClient::connect(url.to_string()).await {
                Ok(res_client) => {
                    info!(target: LOG_TARGET, "Connected successfully");
                    client = Some(
                        res_client
                            .max_decoding_message_size(MAX_GRPC_MESSAGE_SIZE)
                            .max_encoding_message_size(MAX_GRPC_MESSAGE_SIZE),
                    )
                },
                Err(error) => {
                    error!(target: LOG_TARGET,"Failed to connect to base node: {:?}", error);
                    println!("Failed to connect to base node: {error:?}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                },
            }
        }

        Ok(Self {
            client: client.unwrap(),
        })
    }
}

#[async_trait]
impl NodeClient for BaseNodeClientWrapper {
    async fn get_version(&mut self) -> Result<u64, anyhow::Error> {
        info!(target: LOG_TARGET, "Getting node client version");
        let res = self.client.get_version(tonic::Request::new(Empty {})).await?;
        // dbg!(res);
        Ok(0)
    }

    async fn get_block_template(&mut self) -> Result<NewBlockTemplateResponse, anyhow::Error> {
        info!(target: LOG_TARGET, "Getting node block template");
        let res = self
            .client
            .get_new_block_template(tonic::Request::new({
                NewBlockTemplateRequest {
                    max_weight: 0,
                    algo: Some(PowAlgo {
                        pow_algo: PowAlgos::Sha3x.into(),
                    }),
                }
            }))
            .await?;
        info!(target: LOG_TARGET, "Done getting node block template");
        Ok(res.into_inner())
    }

    async fn get_new_block(&mut self, template: NewBlockTemplate) -> Result<NewBlockResult, anyhow::Error> {
        info!(target: LOG_TARGET, "Getting new block template");
        let res = self.client.get_new_block(tonic::Request::new(template)).await?;
        Ok(NewBlockResult::try_from(res.into_inner())?)
    }

    async fn submit_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        info!(target: LOG_TARGET, "Submitting block");
        // dbg!(&block);
        let res = self.client.submit_block(tonic::Request::new(block)).await?;
        info!(target: LOG_TARGET, "Block submitted: {:?}", res);
        Ok(())
    }

    async fn get_height(&mut self) -> Result<HeightData, anyhow::Error> {
        let res = self.client.get_tip_info(Empty {}).await?;
        let res = res.into_inner();
        if let Some(metadata) = res.metadata {
            Ok(HeightData {
                height: metadata.best_block_height,
                tip_hash: metadata.best_block_hash,
                p2pool_height: 0,
                p2pool_tip_hash: vec![],
            })
        } else {
            Err(anyhow!("missing metadata"))
        }
    }
}

#[async_trait]
pub trait NodeClient {
    async fn get_version(&mut self) -> Result<u64, anyhow::Error>;

    async fn get_height(&mut self) -> Result<HeightData, anyhow::Error>;

    async fn get_block_template(&mut self) -> Result<NewBlockTemplateResponse, anyhow::Error>;

    async fn get_new_block(&mut self, template: NewBlockTemplate) -> Result<NewBlockResult, anyhow::Error>;

    async fn submit_block(&mut self, block: Block) -> Result<(), anyhow::Error>;
}

pub(crate) async fn create_client(
    client_type: ClientType,
    url: &str,
    coinbase_extra: String,
) -> Result<Client, anyhow::Error> {
    info!(target: LOG_TARGET, "Creating node client: {}", url);
    Ok(match client_type {
        ClientType::BaseNode => Client::BaseNode(BaseNodeClientWrapper::connect(url).await?),
        ClientType::Benchmark => Client::Benchmark(BenchmarkNodeClient {}),
        ClientType::P2Pool(wallet_payment_address) => {
            Client::P2Pool(P2poolClientWrapper::connect(url, wallet_payment_address, coinbase_extra).await?)
        },
    })
}

pub(crate) enum Client {
    BaseNode(BaseNodeClientWrapper),
    P2Pool(P2poolClientWrapper),
    Benchmark(BenchmarkNodeClient),
}

pub enum ClientType {
    BaseNode,
    Benchmark,
    P2Pool(TariAddress),
}

#[derive(Debug)]
pub struct NewBlockResult {
    pub result: GetNewBlockResult,
    pub target_difficulty: u64,
}

impl TryFrom<GetNewBlockResult> for NewBlockResult {
    type Error = anyhow::Error;

    fn try_from(result: GetNewBlockResult) -> Result<Self, Self::Error> {
        let target_difficulty = result
            .miner_data
            .clone()
            .ok_or(anyhow!("missing miner data"))?
            .target_difficulty;
        Ok(Self {
            result,
            target_difficulty,
        })
    }
}

pub(crate) struct HeightData {
    pub height: u64,
    pub tip_hash: Vec<u8>,
    pub p2pool_height: u64,
    pub p2pool_tip_hash: Vec<u8>,
}

impl Client {
    pub async fn get_version(&mut self) -> Result<u64, anyhow::Error> {
        match self {
            Client::BaseNode(client) => client.get_version().await,
            Client::Benchmark(client) => client.get_version().await,
            Client::P2Pool(client) => client.get_version().await,
        }
    }

    pub async fn get_height(&mut self) -> Result<HeightData, anyhow::Error> {
        match self {
            Client::BaseNode(client) => client.get_height().await,
            Client::Benchmark(client) => client.get_height().await,
            Client::P2Pool(client) => client.get_height().await,
        }
    }

    pub async fn get_block_template(&mut self) -> Result<NewBlockTemplateResponse, anyhow::Error> {
        let timer = Instant::now();
        match self {
            Client::BaseNode(client) => client.get_block_template().await,
            Client::Benchmark(client) => client.get_block_template().await,
            Client::P2Pool(client) => client.get_block_template().await,
        }
        .map(|res| {
            if timer.elapsed() > Duration::from_secs(5) {
                warn!(target: LOG_TARGET, "⚠ SLOW GET_BLOCK_TEMPLATE: Get_block_template took {:?}. Target is 5 seconds", timer.elapsed());
            }
            res
        })
    }

    pub async fn get_new_block(&mut self, template: NewBlockTemplate) -> Result<NewBlockResult, anyhow::Error> {
        let timer = Instant::now();
        match self {
            Client::BaseNode(client) => client.get_new_block(template).await,
            Client::Benchmark(client) => client.get_new_block(template).await,
            Client::P2Pool(client) => client.get_new_block(template).await,
        }
        .map(|res| {
            if timer.elapsed() > Duration::from_secs(5) {
                warn!(target: LOG_TARGET, "⚠ SLOW GET_NEW_BLOCK: Get_new_block took {:?}. Target is 5 seconds", timer.elapsed());
            }
            res
        })
    }

    pub async fn submit_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        let timer = Instant::now();
        let res = match self {
            Client::BaseNode(client) => client.submit_block(block).await,
            Client::Benchmark(client) => client.submit_block(block).await,
            Client::P2Pool(client) => client.submit_block(block).await,
        };
        if timer.elapsed() > Duration::from_secs(5) {
            error!(target: LOG_TARGET, "⚠ SLOW SUBMIT: Submit_block took {:?}. Target is 5 seconds", timer.elapsed());
        }
        res
    }
}

pub(crate) struct BenchmarkNodeClient {}

#[async_trait]
impl NodeClient for BenchmarkNodeClient {
    async fn get_version(&mut self) -> Result<u64, anyhow::Error> {
        Ok(0)
    }

    async fn get_block_template(&mut self) -> Result<NewBlockTemplateResponse, anyhow::Error> {
        todo!()
    }

    async fn get_new_block(&mut self, template: NewBlockTemplate) -> Result<NewBlockResult, anyhow::Error> {
        todo!()
    }

    async fn submit_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn get_height(&mut self) -> Result<HeightData, anyhow::Error> {
        todo!()
    }
}

pub(crate) struct Job {
    pub target_difficulty: u64,
    pub inverted_difficulty: u64,
    pub mining_hash: FixedHash,
    pub job_id: String,
    pub nonce_start: u64,
}

pub(crate) trait JobClient {
    fn get_job(&self) -> Result<Job, anyhow::Error>;
    fn submit(&self, job_id: String, nonce: u64) -> Result<(), anyhow::Error>;
}

pub(crate) struct NodeJobClient {}

impl JobClient for NodeJobClient {
    fn get_job(&self) -> Result<Job, anyhow::Error> {
        // (u64::MAX / num_threads) * thread_index as u64;
        /// targetdif = (u64::MAX / (target_difficulty)).to_le(),
        todo!()
    }

    fn submit(&self, job_id: String, nonce: u64) -> Result<(), anyhow::Error> {
        // header.nonce = nonce.unwrap();

        // let mut mined_block = block.clone();
        // mined_block.header = Some(grpc_header::from(header));
        // let clone_client = node_client.clone();
        // match runtime.block_on(async {
        //     let mut client = clone_client.write().await;
        //     tokio::time::timeout(
        //         std::time::Duration::from_secs(config.template_timeout_secs),
        //         client.submit_block(mined_block),
        //     )
        //     .await?
        // }) {
        //     Ok(_) => {
        //         // stats_store.inc_accepted_blocks();
        //         println!("Block submitted");
        //     },
        //     Err(e) => {
        //         // stats_store.inc_rejected_blocks();
        //         println!("Error submitting block: {:?}", e);
        //     },
        // }

        todo!();
    }
}

pub(crate) fn create_job_client(config: &ConfigFile) -> Result<Box<dyn JobClient>, anyhow::Error> {
    if config.use_stratum {
        let client = NicehashStratumClient {
            url: config.tari_node_url.clone(),
            wallet: config.tari_address.clone(),
            agent: "SRBMiner-MULTI/2.8.7".to_string(),
        };
        return Ok(Box::new(client));
    } else {
        todo!();
        // let client = NodeJobClient {};
        // Ok(Box::new(client))
    }
    // let runtime = Runtime::new()?;
    // let client_type = if benchmark {
    //     ClientType::Benchmark
    // } else if config.p2pool_enabled {
    //     ClientType::P2Pool(TariAddress::from_str(config.tari_address.as_str())?)
    // } else {
    //     ClientType::BaseNode
    // };
    // let mut template_fetch_failures = 0;
    // let coinbase_extra = config.coinbase_extra.clone();
    // let node_client = Arc::new(RwLock::new(runtime.block_on(async move {
    //     node_client::create_client(client_type, &tari_node_url, coinbase_extra).await
    // })?));
    todo!()
}

pub(crate) struct NicehashStratumClient {
    url: String,
    wallet: String,
    agent: String,
    // client: NicehashStratumClient,
    // config: ConfigFile,
}

impl JobClient for NicehashStratumClient {
    fn get_job(&self) -> Result<Job, anyhow::Error> {
        let tcp_stream = TcpStream::connect(&self.url)?;
        let mut writer = tcp_stream.try_clone().unwrap();
        let mut reader = BufReader::new(tcp_stream.try_clone().unwrap());
        let msg = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "login",
            "params": {
                "login": self.wallet,
                "pass": "x",
                "agent": self.agent
            }
        })
        .to_string() +
            "\n";
        writer.write_all(msg.as_bytes())?;
        for line in reader.lines() {
            let line = line?;
            println!("<< {}", line);

            let job_value = serde_json::from_str::<serde_json::Value>(&line);
            let mut job = Job {
                target_difficulty: 0,
                inverted_difficulty: 0,
                mining_hash: FixedHash::zero(),
                job_id: String::new(),
                nonce_start: 0,
            };
            if let Ok(job_value) = job_value {
                if let Some(res) = job_value.get("result") {
                    let id = job_value.get("id");
                    job.job_id = id
                        .unwrap_or(&serde_json::Value::Null)
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    if let Some(j) = res.get("job") {
                        if let Some(target_difficulty) = j.get("target") {
                            let hex = target_difficulty.as_str().unwrap();
                            let mut target_u64 = u64::from_str_radix(hex, 16).unwrap();

                            // target_u64 = u64::from_le(target_u64);

                            job.inverted_difficulty = target_u64;
                            job.target_difficulty = u64::MAX / target_u64;
                        }
                        if let Some(mining_hash) = j.get("blob") {
                            let hex = mining_hash.as_str().unwrap();
                            let vec_blob = hex::decode(hex).unwrap();
                            let mining_hash = FixedHash::try_from(vec_blob.as_slice()).unwrap();
                            job.mining_hash = mining_hash;
                        }
                        if let Some(nonce_start) = j.get("xn") {
                            let hex = nonce_start.as_str().unwrap();
                            let nonce_start = u16::from_str_radix(hex, 16).unwrap();
                            let nonce_start = u64::from(nonce_start);
                            let nonce_start = nonce_start << 16;

                            job.nonce_start = nonce_start;
                        }
                    }
                }
                return Ok(job);
            }
            // if line.contains("mining.notify") {
            // println!("🚀 Mining job received!");
            // }
        }
        todo!()
    }

    fn submit(&self, job_id: String, nonce: u64) -> Result<(), anyhow::Error> {
        todo!()
    }
}
