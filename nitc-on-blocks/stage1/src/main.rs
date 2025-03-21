use std::collections::HashMap;
use std::io::{self, BufRead};
use sha3::{Digest, Sha3_256};
use hex;

// transcation struct to represenmt a blockchain transaction
#[derive(Debug, Clone)]
struct Transaction {
    from: char,
    to: char,
    amount: i32,
    temp: i32,
}

// block struct to represenmt a blocks in the blockchain
#[derive(Debug)]
struct Block {
    block_number: i32,
    merkle_root: String,
    block_hash: String,
    transactions: Vec<Transaction>,
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
        let hash_input: String = format!("{}{}{}", transaction.from, transaction.to, transaction.amount);
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
fn create_block(block_number: i32, prev_block_hash: &str, transactions: Vec<Transaction>) -> Block {
    // calculate the merkle root from transactions
    let merkle_root: String = get_merkle_root(&transactions);
    
    // create the block hash by combining previous hash, block number, and merkle root
    let hash_input: String = format!("{}{}{}", prev_block_hash, block_number, merkle_root);
    let block_hash: String = calculate_hash(hash_input.as_bytes());
    
    // create and return the new block
    Block {
        block_number,
        merkle_root,
        block_hash,
        transactions,
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
            temp: parts[3].parse().unwrap(),
        };
        
        all_transactions.push(transaction);
    }

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
            
            // create a new block when we have 3 transactions or it's the last transaction
            if current_block_txns.len() == 3 {
                let block = create_block(block_number, &prev_block_hash, current_block_txns);
                prev_block_hash = block.block_hash.clone();
                blocks.push(block);
                block_number += 1;
                current_block_txns = Vec::new();
            }
        }
    }

    // create final block with any remaining transactions
    if !current_block_txns.is_empty() {
        let block: Block = create_block(block_number, &prev_block_hash, current_block_txns);
        blocks.push(block);
    }
    
    // print the output
    for block in blocks {
        println!("{}", block.block_number);
        println!("{}", block.block_hash);
        
        // ormat transactions for output
        let txns_str: Vec<String> = block.transactions
            .iter()
            .map(|t: &Transaction| format!("['{}', '{}', {}, {}]", t.from, t.to, t.amount, t.temp))
            .collect();
        
        println!("[{}]", txns_str.join(", "));
        println!("{}", block.merkle_root);
    }
}
