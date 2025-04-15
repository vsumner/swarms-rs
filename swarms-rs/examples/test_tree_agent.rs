use swarms_rs::agent::{TreeAgent,Tree,ForestSwarm};
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let model = "deepseek-chat";
    
    let agent1 = vec![TreeAgent::new(model,"Stock Analysis Agent", "Stock Analysis Agent"),
        TreeAgent::new(model,"Financial Planning Agent", "Financial Planning Agent"),
        TreeAgent::new(model,"Retirement Strategy Agent", "Retirement Strategy Agent")
    ];
    let agent2 = vec![TreeAgent::new(model,"Tax Filing Agent", "Tax Filing Agent")
        ,TreeAgent::new(model,"Investment Strategy Agent", "Investment Strategy Agent"),
        TreeAgent::new(model,"ROTH IRA Agent", "ROTH IRA Agent")];

    let tree = Tree::new("Financial Tree", agent1);
    let tree2 = Tree::new("Investment Tree", agent2);

    let forest = ForestSwarm::new(vec![tree,tree2]);

    let result = forest.run("Our company is incorporated in delaware, how do we do our taxes for free?").await.unwrap();
    println!("Task Result: {}", result);
}
