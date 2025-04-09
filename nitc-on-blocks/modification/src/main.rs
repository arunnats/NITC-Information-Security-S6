// Import necessary libraries
use std::collections::HashMap;
use std::io::{self, BufRead};
use sha3::{Digest, Sha3_256};
use hex;

// Define the Miner struct to represent a miner in the blockchain
#[derive(Debug, Clone)]
struct Miner {
    id: char,                           // Unique identifier for the miner
    computation_score: i32,             // Miner's computational power
    block_hash_score_array: Vec<i32>,   // Array of block hash scores for different block numbers
}

// Define the Transaction struct to represent a transaction in the blockchain
#[derive(Debug, Clone)]
struct Transaction {
    from: char,    // Sender's account
    to: char,      // Receiver's account
    amount: i32,   // Amount to be transferred
    incentive: i32, // Transaction fee or incentive
}

// Define the Block struct to represent a block in the blockchain
#[derive(Debug)]
struct Block {
    block_number: i32,      // Unique number for the block
    merkle_root: String,    // Root hash of the Merkle tree of transactions
    block_hash: String,     // Hash of the entire block
    transactions: Vec<Transaction>, // List of transactions in the block
    nonce: i32,             // Nonce used for Proof of Work
    miner_id: char          // ID of the miner who mined this block
}

// Define the MerkleNode struct to represent a node in the Merkle tree
#[derive(Debug, Clone)]
struct MerkleNode {
    hash: String,                   // Hash of this node
    left: Option<Box<MerkleNode>>,  // Left child node
    right: Option<Box<MerkleNode>>, // Right child node
}

// Implement methods for MerkleNode
impl MerkleNode {
    // Create a new leaf node from a transaction
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

    // Create a new internal node from two child nodes
    fn new_node(left: MerkleNode, right: MerkleNode) -> MerkleNode {
        let combined: String = format!("{}{}", left.hash, right.hash);
        let hash: String = calculate_hash(combined.as_bytes());

        MerkleNode { 
            hash: hash, 
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}

// Function to calculate SHA3-256 hash of given data
pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// Function to build a Merkle tree from a list of transactions
fn build_merkle_tree(transactions: &[Transaction]) -> Option<MerkleNode> {
    if transactions.is_empty() {
        return None;
    }

    // Create leaf nodes for all transactions
    let mut nodes: Vec<MerkleNode> = transactions
    .iter()
    .map(|txn| MerkleNode::new_leaf(txn))
    .collect();

    // Build the tree bottom-up
    while nodes.len() > 1 {
        let mut new_level: Vec<MerkleNode> = Vec::new();

        for chunk in nodes.chunks(2) {
            if chunk.len() == 2 {
                new_level.push(MerkleNode::new_node(chunk[0].clone(), chunk[1].clone()));
            } else {
                 new_level.push(MerkleNode::new_node(chunk[0].clone(), chunk[0].clone()));
            }
        }

        nodes = new_level;
    }

    Some(nodes.remove(0))
}

// Function to get the Merkle root hash from a list of transactions
fn get_merkle_root(transactions: &[Transaction]) -> String {
    match build_merkle_tree(transactions) {
        Some(root) => root.hash,
        None => String::from("0")
    }
}

// Function to create a new block
fn create_block(block_number: i32, prev_block_hash: &str, transactions: Vec<Transaction>, miners: &[Miner]) -> Block {
    let merkle_root: String = get_merkle_root(&transactions);
    let hash_input: String = format!("{}{}{}", prev_block_hash, block_number, merkle_root);
    let block_hash: String = calculate_hash(hash_input.as_bytes());
    let mut nonce = 0;

    // Perform Proof of Work
    loop {
        let pow_input = format!("{}{}", block_hash, nonce);
        let computed_hash = calculate_hash(pow_input.as_bytes());

        if computed_hash.chars().last().unwrap() == '0' {
            break;
        }
        
        nonce += 1;
    }

    // Select the miner for this block
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

// Function to execute a transaction
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

// Main function
fn main() {
    let stdin: io::Stdin = io::stdin();
    let mut lines: io::Lines<io::StdinLock<'_>> = stdin.lock().lines();

    // Read number of accounts
    let num_accounts: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // Read account balances
    let mut balances: HashMap<char, i32> = HashMap::new();
    for _ in 0..num_accounts {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let account: char = parts[0].chars().next().unwrap();
        let balance: i32 = parts[1].parse().unwrap();
        balances.insert(account, balance);
    }

    // Read number of transactions
    let num_transactions: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // Read transactions
    let mut all_transactions: Vec<Transaction> = Vec::new();
    for _ in 0..num_transactions {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let transaction: Transaction = Transaction {
            from: parts[0].chars().next().unwrap(),
            to: parts[2].chars().next().unwrap(),
            amount: parts[3].parse().unwrap(),
            incentive: parts[1].parse().unwrap(),
        };
        
        all_transactions.push(transaction);
    }

    // Read block reward
    let block_reward: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // Read number of miners
    let num_miners: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // Read miner information
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

    // For some stupid reason we arent sorting it anymore
    // Sort transactions by incentive (descending) and receiver (ascending)
    // all_transactions.sort_by(|a: &Transaction, b: &Transaction| {
    //     let incentive_cmp = b.incentive.cmp(&a.incentive);
        
    //     if incentive_cmp == std::cmp::Ordering::Equal {
    //         a.to.cmp(&b.to)
    //     } else {
    //         incentive_cmp
    //     }
    // });

    // Process transactions and create blocks
    let mut blocks: Vec<Block> = Vec::new();
    let mut block_number: i32 = 1;
    let mut prev_block_hash: String = String::from("0");
    let mut current_block_txns: Vec<Transaction> = Vec::new();
    
    for txn in all_transactions {
        if execute_transaction(&txn, &mut balances) {
            current_block_txns.push(txn);
            
            if current_block_txns.len() == 10 {
                let block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
                *balances.entry(block.miner_id).or_insert(0) += block_reward;
                prev_block_hash = block.block_hash.clone();
                blocks.push(block);
                block_number += 1;
                current_block_txns = Vec::new();
            }
        }
    }

    // Create final block if there are remaining transactions
    if !current_block_txns.is_empty() {
        let block: Block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
        *balances.entry(block.miner_id).or_insert(0) += block_reward;
        blocks.push(block);
    }
    
    // Print the results
    for block in blocks {
        println!("{}", block.block_number);
        println!("{}", block.block_hash);
        
        let txns_str: Vec<String> = block.transactions
            .iter()
            .map(|t: &Transaction| format!("['{}', '{}', {}, {}]", t.from, t.amount, t.to, t.incentive))
            .collect();
        
        println!("[{}]", txns_str.join(", "));
        println!("{}", block.merkle_root);
        println!("{} {}", block.nonce, block.miner_id);
    }
}