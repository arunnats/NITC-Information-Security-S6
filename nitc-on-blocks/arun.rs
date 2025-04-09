use std::collections::HashMap;
use std::io::{self, BufRead};
use sha3::{Digest, Sha3_256};
use hex;

#[derive(Debug, Clone)]
struct Miner {
    id: char,
    computation_score: i32,
    block_hash_score_array: Vec<i32>,
}

#[derive(Debug, Clone)]
struct Transaction {
    from: char,
    to: char,
    amount: i32,
    incentive: i32,
}

#[derive(Debug)]
struct Block {
    block_number: i32,
    merkle_root: String,
    block_hash: String,
    transactions: Vec<Transaction>,
    nonce: i32,
    miner_id: char
}

#[derive(Debug, Clone)]
struct MerkleNode {
    hash: String,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    fn new_leaf(transaction: &Transaction) -> MerkleNode {
        let hash_input: String = format!("{}{}{}{}", 
            transaction.from, 
            transaction.incentive, 
            transaction.to, 
            transaction.amount
        );
        let hash: String = calculate_hash(hash_input.as_bytes());
        
        MerkleNode { 
            hash: hash, 
            left: None, 
            right: None 
        }
    }

    fn new_node(left: MerkleNode, right: Option<MerkleNode>) -> MerkleNode {
        let hash: String = match &right {
            Some(right_node) => {
                let combined: String = format!("{}{}", left.hash, right_node.hash);
                calculate_hash(combined.as_bytes())
            },
            None => {
                left.hash.clone()
            }
        };

        MerkleNode { 
            hash: hash, 
            left: Some(Box::new(left)),
            right: (right.map(Box::new))
        }
    }
}

pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn build_merkle_tree(transactions: &[Transaction]) -> Option<MerkleNode> {
    if transactions.is_empty() {
        return None;
    }

    let mut nodes: Vec<MerkleNode> = transactions
    .iter()
    .map(|txn| MerkleNode::new_leaf(txn))
    .collect();

    while nodes.len() > 1 {
        let mut new_level: Vec<MerkleNode> = Vec::new();

        for chunk in nodes.chunks(2) {
            if chunk.len() == 2 {
                new_level.push(MerkleNode::new_node(chunk[0].clone(), Some(chunk[1].clone())));
            } else {
                new_level.push(MerkleNode::new_node(chunk[0].clone(), None));
            }
        }

        nodes = new_level;
    }

    Some(nodes.remove(0))
}

fn get_merkle_root(transactions: &[Transaction]) -> String {
    match build_merkle_tree(transactions) {
        Some(root) => root.hash,
        None => String::from("0")
    }
}

fn create_block(block_number: i32, prev_block_hash: &str, transactions: Vec<Transaction>, miners: &[Miner]) -> Block {
    let merkle_root: String = get_merkle_root(&transactions);
    let hash_input: String = format!("{}{}{}", prev_block_hash, block_number, merkle_root);
    let block_hash: String = calculate_hash(hash_input.as_bytes());
    let mut nonce = 0;

    loop {
        let pow_input = format!("{}{}", block_hash, nonce);
        let computed_hash = calculate_hash(pow_input.as_bytes());

        if computed_hash.chars().last().unwrap() == '0' {
            break;
        }
        
        nonce += 1;
    }

    let block_index: usize = (block_number % 8) as usize;
    let mut max_score: i32 = -1;
    let mut selected_miner: char = ' ';

    for miner in miners {
        let block_hash_score = miner.block_hash_score_array[block_index];
        let block_sealing_score = miner.computation_score * block_hash_score;
        
        if block_sealing_score > max_score {
            max_score = block_sealing_score;
            selected_miner = miner.id;
        }
    }

    Block {
        block_number,
        merkle_root,
        block_hash,
        transactions,
        nonce,
        miner_id: selected_miner
    }
}

fn execute_transaction(transaction: &Transaction, balances: &mut HashMap<char, i32>) -> bool {
    let from_balance: i32 = *balances.get(&transaction.from).unwrap_or(&0);
    
    if from_balance >= transaction.amount {
        *balances.entry(transaction.from).or_insert(0) -= transaction.amount;
        *balances.entry(transaction.to).or_insert(0) += transaction.amount;
        true
    } else {
        false
    }
}

fn main() {
    let stdin: io::Stdin = io::stdin();
    let mut lines: io::Lines<io::StdinLock<'_>> = stdin.lock().lines();

    let num_accounts: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let mut balances: HashMap<char, i32> = HashMap::new();
    for _ in 0..num_accounts {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let account: char = parts[0].chars().next().unwrap();
        let balance: i32 = parts[1].parse().unwrap();
        balances.insert(account, balance);
    }

    let num_transactions: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let mut all_transactions: Vec<Transaction> = Vec::new();
    for _ in 0..num_transactions {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let transaction: Transaction = Transaction {
            from: parts[0].chars().next().unwrap(),
            to: parts[1].chars().next().unwrap(),
            amount: parts[2].parse().unwrap(),
            incentive: parts[3].parse().unwrap(),
        };
        
        all_transactions.push(transaction);
    }

    let block_reward: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let num_miners: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let mut miners: Vec<Miner> = Vec::new();
    for _ in 0..num_miners {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let id: char = parts[0].chars().next().unwrap();
        let computation_score: i32 = parts[1].parse().unwrap();
        
        let mut block_hash_score_array: Vec<i32> = Vec::new();
        for i in 2..10 {
            block_hash_score_array.push(parts[i].parse().unwrap());
        }
        
        miners.push(Miner {
            id,
            computation_score,
            block_hash_score_array,
        });
    }

    all_transactions.sort_by(|a: &Transaction, b: &Transaction| {
        let incentive_cmp = b.incentive.cmp(&a.incentive);
        
        if incentive_cmp == std::cmp::Ordering::Equal {
            a.to.cmp(&b.to)
        } else {
            incentive_cmp
        }
    });

    let mut blocks: Vec<Block> = Vec::new();
    let mut block_number: i32 = 1;
    let mut prev_block_hash: String = String::from("0");
    let mut current_block_txns: Vec<Transaction> = Vec::new();
    
    for txn in all_transactions {
        if execute_transaction(&txn, &mut balances) {
            current_block_txns.push(txn);
            
            if current_block_txns.len() == 4 {
                let block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
                *balances.entry(block.miner_id).or_insert(0) += block_reward;
                prev_block_hash = block.block_hash.clone();
                blocks.push(block);
                block_number += 1;
                current_block_txns = Vec::new();
            }
        }
    }

    if !current_block_txns.is_empty() {
        let block: Block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
        *balances.entry(block.miner_id).or_insert(0) += block_reward;
        blocks.push(block);
    }
    
    for block in blocks {
        println!("{}", block.block_number);
        println!("{}", block.block_hash);
        
        let txns_str: Vec<String> = block.transactions
            .iter()
            .map(|t: &Transaction| format!("['{}', '{}', {}, {}]", t.from, t.to, t.amount, t.incentive))
            .collect();
        
        println!("[{}]", txns_str.join(", "));
        println!("{}", block.merkle_root);
        println!("{} {}", block.nonce, block.miner_id);
    }
}