use serde::{Deserialize, Serialize};

/// 单个 Meme Agent 的配置 (对应 MemeAgentConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeAgentConfig {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
}

/// Meme Swarm 的完整配置 (对应 MemeSwarmConfig)
#[derive(Debug, Serialize, Deserialize)]
pub struct MemeSwarmConfig {
    pub name: String,
    pub description: String,
    pub agents: Vec<MemeAgentConfig>,
    pub max_loops: u32,
}

struct MemeAgentGenerator{
    
}

impl MemeAgentGenerator {

    pub fn new(name:&str,description:&str,system_prompt:&str){  
        
    }

    pub fn run(task:&str){

    }

    pub fn swarm_router(){

    }


}