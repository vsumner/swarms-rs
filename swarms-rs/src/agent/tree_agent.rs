use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};
use crate::{llm::provider::openai::OpenAI, structs::agent::Agent,structs::agent::AgentError};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeAgent {
    pub model:String,
    pub agent_name: String,
    pub system_prompt: String,
}
impl TreeAgent {
    pub fn new(model:&str,agent_name: &str, system_prompt: &str) -> Self {
        TreeAgent {
            model: model.to_string(),
            agent_name: agent_name.to_string(),
            system_prompt: system_prompt.to_string(),
        }
    }

    pub async fn run(&self, prompt: &str) ->  Result<String, AgentError> {
        let client = match self.model.as_str() {
            "deepseek-chat" => {
                let base_url = std::env::var("DEEPSEEK_BASE_URL").unwrap();
                let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap();
                OpenAI::from_url(base_url, api_key).set_model(self.model.clone())
            },
            _ => {
                let base_url = std::env::var("OPENAI_BASE_URL").unwrap();
                let api_key = std::env::var("OPENAI_API_KEY").unwrap();
                OpenAI::from_url(base_url, api_key).set_model(self.model.clone())
            }
        };
        // Create agent
        let agent = client
            .agent_builder()
            .system_prompt(self.system_prompt.clone())
            .agent_name(self.agent_name.clone())
            .user_name("TestUser")
            .enable_autosave()
            .max_loops(1)
            .save_state_dir(format!("./temp/test_tree_agent_state_{}.json",Uuid::new_v4().to_string()))
            .enable_plan("Split the task into subtasks.".to_owned())
            .build();
        println!("call model:{} ,chat prompt:{}",&self.agent_name,&prompt);
        // Execute task and return result
        let response = agent.run(prompt.to_string()).await;
        match response {
            Ok(result) => {
                Ok(result)
            }
            Err(err) => {
                Err(err)
            }
        }
    }


}
    
pub struct Tree {
    pub tree_name: String,
    pub agents: Vec<TreeAgent>,
}


impl Tree {
    pub fn new(tree_name: &str, agents: Vec<TreeAgent>) -> Self {
        let tree = Tree {
            tree_name: tree_name.to_string(),
            agents:agents,
        };
        tree
    }

    pub fn find_relevant_agent(&self, task: &str) -> Option<&TreeAgent> {
        println!("Finding relevant agent for task: '{}'", task); 
        //TODO 判断当前的 system_prompt 和 task 选择最相关 , python 实现sentence_transformers all-MiniLM-L6-v2

        //临时随机
        let mut rng = rand::thread_rng();
        self.agents.choose(&mut rng)
    }

}



pub struct ForestSwarm {
    pub trees: Vec<Tree>,
    pub save_file_path: String,
}

impl ForestSwarm {
    pub fn new(trees: Vec<Tree>) -> Self {
        let save_file_path = format!("forest_swarm_{}.json", Uuid::new_v4().to_string());
        ForestSwarm {
            trees,
            save_file_path,
        }
    }

    pub fn find_relevant_tree(&self, task: &str) -> Option<&Tree> {
        println!("Searching for the most relevant tree for task: {}", task);
        //TODO 根据tree name 和task 选择最相关
        //self.trees.iter().find(|tree| tree.find_relevant_agent(task).is_some())

        //随机一个
        let mut rng = rand::thread_rng();
        self.trees.choose(&mut rng)
    }


    pub async fn run(&self, task: &str) -> Result<String, AgentError> {
        println!("Running task across ForestSwarm: {} ", task);
        match self.find_relevant_tree(task) {
            Some(tree) => {
                match tree.find_relevant_agent(task) {
                    Some(agent) => {
                        match agent.run(task).await {
                            Ok(result) => Ok(result),
                            Err(err) => {
                                println!("Error occurred while running agent: {:?}", err);
                                Err(AgentError::NoChoiceFound)
                            }
                        }
                    }
                    None => {
                        println!("No relevant agent found in selected tree.");
                        Err(AgentError::NoChoiceFound)
                    }
                }
            }
            None => {
                println!("No relevant tree found.");
                Err(AgentError::NoChoiceFound)
            }
        }
    }
    
}
