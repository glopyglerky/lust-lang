// Rc<RefCell<T>> fixture set: accounts shared through a HashMap, transfers with two borrows, ptr_eq, and Default.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
struct Account {
    owner: String,
    balance: i64,
    log: Vec<String>,
}

type Shared = Rc<RefCell<Account>>;

fn transfer(from: &Shared, to: &Shared, amount: i64) -> Result<(), String> {
    if Rc::ptr_eq(from, to) {
        return Err("same account".to_string());
    }
    let mut f = from.borrow_mut();
    if f.balance < amount {
        return Err(format!("{} is short", f.owner));
    }
    f.balance -= amount;
    f.log.push(format!("-{}", amount));
    let mut t = to.borrow_mut();
    t.balance += amount;
    t.log.push(format!("+{}", amount));
    Ok(())
}

fn main() {
    let mut bank: HashMap<String, Shared> = HashMap::new();
    for (name, start) in [("ann", 100), ("bob", 20), ("cy", 0)] {
        bank.insert(name.to_string(), Rc::new(RefCell::new(Account { owner: name.to_string(), balance: start, log: vec![] })));
    }
    let ann = bank["ann"].clone();
    let bob = Rc::clone(&bank["bob"]);
    println!("{:?}", transfer(&ann, &bob, 30));
    println!("{:?}", transfer(&bob, &bank["cy"], 100));
    println!("{:?}", transfer(&ann, &ann, 1));
    let mut names: Vec<&String> = bank.keys().collect();
    names.sort();
    for n in names {
        let acc = bank[n].borrow();
        println!("{} {} {:?} {}", acc.owner, acc.balance, acc.log, Rc::strong_count(&bank[n]));
    }
    let total: i64 = bank.values().map(|a| a.borrow().balance).sum();
    println!("{}", total);
    let fresh: Shared = Default::default();
    fresh.borrow_mut().owner.push_str("new");
    println!("{:?}", fresh);
}
