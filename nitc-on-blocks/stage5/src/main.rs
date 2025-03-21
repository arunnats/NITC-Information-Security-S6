use std::collections::HashMap;
use std::io::{self, BufRead};
use sha3::{Digest, Sha3_256};
use hex;

// miner structure
#[derive(Debug, Clone)]
struct Miner {
    id: String,
    computation_score: i32,
    block_hash_score_array: Vec<i32>,
}


// transcation struct to represenmt a blockchain transaction
#[derive(Debug, Clone)]
struct Transaction {
    from: char,
    to: char,
    amount: i32,
    incentive: i32,
}

// block struct to represenmt a blocks in the blockchain
#[derive(Debug)]
struct Block {
    block_number: i32,
    merkle_root: String,
    block_hash: String,
    transactions: Vec<Transaction>,
    nonce: i32,
    miner_id: String
}

// block MerkleNode struct for building the merkle tree
#[derive(Debug, Clone)]
struct MerkleNode {
    hash: String, // hash value of string
    left: Option<Box<MerkleNode>>, // optional left node
    right: Option<Box<MerkleNode>>, // optional rightt node
}

impl MerkleNode {
    // new leaf node function -> take the transaction as input and store its hash (from+to+amount)
    fn new_leaf(transaction: &Transaction) -> MerkleNode {
        // hash format: sender + incentive + receiver + amount
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

    // mew merkle node function -> take the two children (right child need not be present) as inpur and create the node
    fn new_node(left: MerkleNode, right: Option<MerkleNode>) -> MerkleNode {

        // hash is combined hash of the childen or 
        let hash: String = match &right {
            Some(right_node) => {
                // Two children
                let combined: String = format!("{}{}", left.hash, right_node.hash);
                calculate_hash(combined.as_bytes())
            },
            None => {
                // single child -> we try to build the tree left skewed
                left.hash.clone()
            }
        };

        MerkleNode { 
            hash: hash, 
            left: Some(Box::new(left)), // store left child
            right: (right.map(Box::new)) // store right child if exits
        }
    }
}

// create a Sha3_256 hasher and hash the data input to it, return the final hex as a string
pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data); // add data to the hasher
    hex::encode(hasher.finalize()) // get final hash and convert to hex string
}

// build the merkle tree from list of transactions
fn build_merkle_tree(transactions: &[Transaction]) -> Option<MerkleNode> {
    if transactions.is_empty() {
        return None;
    }

    // get final hash and convert to hex string
    let mut nodes: Vec<MerkleNode> = transactions
    .iter()
    .map(|txn| MerkleNode::new_leaf(txn))
    .collect();

    // iteratively build the tree by combining nodes at each level
    // continue until we have just one node (the root)
    while nodes.len() > 1 {
        let mut new_level: Vec<MerkleNode> = Vec::new();

        // process nodes in pairs
        for chunk in nodes.chunks(2) {
            // if we have a pair, create a node with two children
            if chunk.len() == 2 {
                new_level.push(MerkleNode::new_node(chunk[0].clone(), Some(chunk[1].clone())));
            } else {
                // if we have a single node left, create a node with one child
                new_level.push(MerkleNode::new_node(chunk[0].clone(), None));
            }
        }

        // replace current level with the new level for next iteration
        nodes = new_level;
    }

    // return the root
    Some(nodes.remove(0))
}

// get the merkle root hash from a list of transactions
// returns "0" for empty transaction list
fn get_merkle_root(transactions: &[Transaction]) -> String {
    match build_merkle_tree(transactions) {
        Some(root) => root.hash,
        None => String::from("0")
    }
}

// create new block with the given transactions and previous block information
fn create_block(block_number: i32, prev_block_hash: &str, transactions: Vec<Transaction>, miners: &[Miner]) -> Block {
    // calculate the merkle root from transactions
    let merkle_root: String = get_merkle_root(&transactions);
    
    // create the block hash by combining previous hash, block number, and merkle root
    let hash_input: String = format!("{}{}{}", prev_block_hash, block_number, merkle_root);
    let block_hash: String = calculate_hash(hash_input.as_bytes());
    
    // find the minimum nonce that satisfies the PoW condition
    let mut nonce = 0;
    loop {
        let pow_input = format!("{}{}", block_hash, nonce);
        let computed_hash = calculate_hash(pow_input.as_bytes());
        
        // check if the last digit of the computed hash is '0'
        if computed_hash.chars().last().unwrap() == '0' {
            break;
        }
        
        nonce += 1;
    }

    // select the miner with the highest block sealing score
    let block_index = (block_number % 8) as usize;
    let mut max_score = -1;
    let mut selected_miner = String::new();

    for miner in miners {
        let block_hash_score = miner.block_hash_score_array[block_index];
        let block_sealing_score = miner.computation_score * block_hash_score;
        
        if block_sealing_score > max_score {
            max_score = block_sealing_score;
            selected_miner = miner.id.clone();
        }
    }

    // create and return the new block
    Block {
        block_number,
        merkle_root,
        block_hash,
        transactions,
        nonce,
        miner_id: selected_miner
    }
}

// execute a transaction by updating account balances
// returns true if transaction was successful, false if sender has insufficient funds
fn execute_transaction(transaction: &Transaction, balances: &mut HashMap<char, i32>) -> bool {
    // get sender's balance (default to 0 if account doesn't exist)
    let from_balance: i32 = *balances.get(&transaction.from).unwrap_or(&0);
    
    // check if sufficient balance -> execute the transaction
    if from_balance >= transaction.amount {
        // Update sender's balance
        *balances.entry(transaction.from).or_insert(0) -= transaction.amount;
        
        // Update receiver's balance
        *balances.entry(transaction.to).or_insert(0) += transaction.amount;
        
        true
    } else {
        false // transaction failed - insufficient funds
    }
}

fn main() {
    let stdin: io::Stdin = io::stdin();
    let mut lines: io::Lines<io::StdinLock<'_>> = stdin.lock().lines();

    // read number of accounts
    let num_accounts: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // read account balances
    let mut balances: HashMap<char, i32> = HashMap::new();
    for _ in 0..num_accounts {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let account: char = parts[0].chars().next().unwrap();
        let balance: i32 = parts[1].parse().unwrap();
        balances.insert(account, balance);
    }

    // read number of transactions
    let num_transactions: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // read all transactions
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

     // read number of miners
     let num_miners: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

     // read miner information
    let mut miners: Vec<Miner> = Vec::new();
    for _ in 0..num_miners {
        let line: String = lines.next().unwrap().unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let id: String = parts[0].to_string();
        let computation_score: i32 = parts[1].parse().unwrap();
        
        let mut block_hash_score_array: Vec<i32> = Vec::new();
        for i in 2..10 {  // 8 values for block_hash_score_array
            block_hash_score_array.push(parts[i].parse().unwrap());
        }
        
        miners.push(Miner {
            id,
            computation_score,
            block_hash_score_array,
        });
    }

    // sort transactions by incentive (descending), then receiver (ascending)
    all_transactions.sort_by(|a: &Transaction, b: &Transaction| {
        // first compare incentives (descending)
        let incentive_cmp = b.incentive.cmp(&a.incentive);
        
        if incentive_cmp == std::cmp::Ordering::Equal {
            // if incentives are equal, compare receivers (ascending)
            a.to.cmp(&b.to)
        } else {
            incentive_cmp
        }
    });

    // process transactions and create blocks
    let mut blocks: Vec<Block> = Vec::new();
    let mut block_number: i32 = 1;
    let mut prev_block_hash: String = String::from("0"); // genesis block starts with "0"
    let mut current_block_txns: Vec<Transaction> = Vec::new();
    
    // process each transaction
    for txn in all_transactions {
        if execute_transaction(&txn, &mut balances) {
            // if transaction is successful, add it to current block
            current_block_txns.push(txn);
            
            // create a new block when we have 4 transactions or it's the last transaction
            if current_block_txns.len() == 4 {
                let block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
                prev_block_hash = block.block_hash.clone();
                blocks.push(block);
                block_number += 1;
                current_block_txns = Vec::new();
            }
        }
    }

    // create final block with any remaining transactions
    if !current_block_txns.is_empty() {
        let block: Block = create_block(block_number, &prev_block_hash, current_block_txns, &miners);
        blocks.push(block);
    }
    
    // print the output
    for block in blocks {
        println!("{}", block.block_number);
        println!("{}", block.block_hash);
        
        // format transactions for output
        let txns_str: Vec<String> = block.transactions
            .iter()
            .map(|t: &Transaction| format!("['{}', '{}', {}, {}]", t.from, t.to, t.amount, t.incentive))
            .collect();
        
        println!("[{}]", txns_str.join(", "));
        println!("{}", block.merkle_root);
        println!("{} {}", block.nonce, block.miner_id);
    }
}
