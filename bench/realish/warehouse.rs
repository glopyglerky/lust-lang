// A small warehouse service: inventory, orders, a pricing-rule language, routing, and reports.
// Written in the lust subset, which is plain Rust. bench/realish/make.py clones this module many
// times (renaming its items) to build a crate of realistic size.

// ---------- Deterministic random numbers ----------

#[derive(Debug, Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng { state: seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state >> 33
    }

    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
}

// ---------- Inventory ----------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Category {
    Hardware,
    Electrical,
    Plumbing,
    Garden,
}

#[derive(Debug, Clone, PartialEq)]
struct Product {
    sku: u32,
    name: String,
    category: Category,
    price_cents: i64,
    weight_grams: i64,
}

#[derive(Debug, Clone)]
struct StockLevel {
    sku: u32,
    on_hand: i64,
    reserved: i64,
    reorder_point: i64,
    reorder_qty: i64,
}

#[derive(Debug)]
struct Inventory {
    products: Vec<Product>,
    stock: Vec<StockLevel>,
    restocks: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum StockError {
    UnknownSku(u32),
    Insufficient { sku: u32, wanted: i64, available: i64 },
}

impl Inventory {
    fn new() -> Inventory {
        Inventory { products: Vec::new(), stock: Vec::new(), restocks: 0 }
    }

    fn add_product(&mut self, sku: u32, name: &str, category: Category, price_cents: i64, weight_grams: i64) {
        self.products.push(Product { sku, name: name.to_string(), category, price_cents, weight_grams });
        let reorder_point = 5 + (sku % 7) as i64;
        self.stock.push(StockLevel { sku, on_hand: 20, reserved: 0, reorder_point, reorder_qty: 30 });
    }

    fn product_index(&self, sku: u32) -> Option<usize> {
        let mut i = 0;
        while i < self.products.len() {
            if self.products[i].sku == sku {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn product(&self, sku: u32) -> Option<&Product> {
        match self.product_index(sku) {
            Some(i) => Some(&self.products[i]),
            None => None,
        }
    }

    fn available(&self, sku: u32) -> i64 {
        match self.product_index(sku) {
            Some(i) => self.stock[i].on_hand - self.stock[i].reserved,
            None => 0,
        }
    }

    fn reserve(&mut self, sku: u32, qty: i64) -> Option<StockError> {
        let i = match self.product_index(sku) {
            Some(i) => i,
            None => return Some(StockError::UnknownSku(sku)),
        };
        let available = self.stock[i].on_hand - self.stock[i].reserved;
        if available < qty {
            return Some(StockError::Insufficient { sku, wanted: qty, available });
        }
        self.stock[i].reserved += qty;
        None
    }

    fn release(&mut self, sku: u32, qty: i64) {
        if let Some(i) = self.product_index(sku) {
            let held = self.stock[i].reserved;
            self.stock[i].reserved -= qty.min(held);
        }
    }

    fn ship(&mut self, sku: u32, qty: i64) {
        if let Some(i) = self.product_index(sku) {
            let level = &mut self.stock[i];
            level.reserved -= qty;
            level.on_hand -= qty;
        }
    }

    /// Restocks every product at or below its reorder point. Returns how many were restocked.
    fn restock(&mut self) -> u32 {
        let mut count = 0;
        for level in &mut self.stock {
            if level.on_hand - level.reserved <= level.reorder_point {
                level.on_hand += level.reorder_qty;
                count += 1;
            }
        }
        self.restocks += count;
        count
    }

    fn total_value_cents(&self) -> i64 {
        let mut total = 0;
        let mut i = 0;
        while i < self.products.len() {
            total += self.products[i].price_cents * self.stock[i].on_hand;
            i += 1;
        }
        total
    }
}

fn category_name(c: Category) -> String {
    let name = match c {
        Category::Hardware => "hardware",
        Category::Electrical => "electrical",
        Category::Plumbing => "plumbing",
        Category::Garden => "garden",
    };
    name.to_string()
}

// ---------- Orders ----------

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderState {
    Pending,
    Reserved,
    Paid,
    Shipped,
    Cancelled,
}

#[derive(Debug, Clone)]
struct OrderLine {
    sku: u32,
    qty: i64,
}

#[derive(Debug, Clone)]
struct Order {
    id: u32,
    customer: String,
    lines: Vec<OrderLine>,
    state: OrderState,
    total_cents: i64,
    destination: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderError {
    Empty,
    BadQuantity(i64),
    Stock(StockError),
    WrongState { from: OrderState, action: String },
}

fn can_transition(from: OrderState, to: OrderState) -> bool {
    match from {
        OrderState::Pending => to == OrderState::Reserved || to == OrderState::Cancelled,
        OrderState::Reserved => to == OrderState::Paid || to == OrderState::Cancelled,
        OrderState::Paid => to == OrderState::Shipped,
        OrderState::Shipped | OrderState::Cancelled => false,
    }
}

fn transition(order: &mut Order, to: OrderState) -> Option<OrderError> {
    if can_transition(order.state, to) {
        order.state = to;
        None
    } else {
        Some(OrderError::WrongState { from: order.state, action: format!("{:?}", to) })
    }
}

fn validate(order: &Order) -> Option<OrderError> {
    if order.lines.is_empty() {
        return Some(OrderError::Empty);
    }
    for line in &order.lines {
        if line.qty <= 0 || line.qty > 50 {
            return Some(OrderError::BadQuantity(line.qty));
        }
    }
    None
}

fn reserve_order(inv: &mut Inventory, order: &mut Order) -> Option<OrderError> {
    if let Some(e) = validate(order) {
        return Some(e);
    }
    let mut done: Vec<OrderLine> = Vec::new();
    for line in &order.lines {
        if let Some(e) = inv.reserve(line.sku, line.qty) {
            for d in &done {
                inv.release(d.sku, d.qty);
            }
            return Some(OrderError::Stock(e));
        }
        done.push(line.clone());
    }
    transition(order, OrderState::Reserved)
}

fn cancel_order(inv: &mut Inventory, order: &mut Order) -> Option<OrderError> {
    let was_reserved = order.state == OrderState::Reserved;
    if let Some(e) = transition(order, OrderState::Cancelled) {
        return Some(e);
    }
    if was_reserved {
        for line in &order.lines {
            inv.release(line.sku, line.qty);
        }
    }
    None
}

fn ship_order(inv: &mut Inventory, order: &mut Order) -> Option<OrderError> {
    if let Some(e) = transition(order, OrderState::Shipped) {
        return Some(e);
    }
    for line in &order.lines {
        inv.ship(line.sku, line.qty);
    }
    None
}

fn describe_error(e: &OrderError) -> String {
    match e {
        OrderError::Empty => "order has no lines".to_string(),
        OrderError::BadQuantity(q) => format!("bad quantity {}", q),
        OrderError::Stock(StockError::UnknownSku(s)) => format!("unknown sku {}", s),
        OrderError::Stock(StockError::Insufficient { sku, wanted, available }) => {
            format!("sku {} short: wanted {}, have {}", sku, wanted, available)
        }
        OrderError::WrongState { from, action } => format!("cannot {} from {:?}", action, from),
    }
}

// ---------- Pricing rules: a tiny expression language ----------
//
// Rules look like `qty * 120 + (weight / 1000) * 35 - discount`, with integer arithmetic.

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Max,
    Comma,
}

fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = collect_chars(src);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let mut n: i64 = 0;
            while i < chars.len() && chars[i].is_ascii_digit() {
                n = n * 10 + chars[i].to_digit(10).unwrap() as i64;
                i += 1;
            }
            tokens.push(Token::Num(n));
        } else if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }
            if word == "max" {
                tokens.push(Token::Max);
            } else {
                tokens.push(Token::Ident(word));
            }
        } else {
            let t = match c {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '(' => Token::LParen,
                ')' => Token::RParen,
                _ => Token::Comma,
            };
            tokens.push(t);
            i += 1;
        }
    }
    tokens
}

fn collect_chars(s: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in s.chars() {
        out.push(c);
    }
    out
}

#[derive(Debug, Clone)]
enum Node {
    Num(i64),
    Var(String),
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Max(usize, usize),
}

/// A parsed rule. Nodes live in a Vec and refer to each other by index.
#[derive(Debug)]
struct Rule {
    nodes: Vec<Node>,
    root: usize,
}

struct RuleParser {
    tokens: Vec<Token>,
    pos: usize,
    nodes: Vec<Node>,
}

impl RuleParser {
    fn peek(&self) -> Option<Token> {
        if self.pos < self.tokens.len() { Some(self.tokens[self.pos].clone()) } else { None }
    }

    fn push(&mut self, n: Node) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }

    fn expr(&mut self) -> usize {
        let mut lhs = self.term();
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.pos += 1;
                    let rhs = self.term();
                    lhs = self.push(Node::Add(lhs, rhs));
                }
                Some(Token::Minus) => {
                    self.pos += 1;
                    let rhs = self.term();
                    lhs = self.push(Node::Sub(lhs, rhs));
                }
                _ => break,
            }
        }
        lhs
    }

    fn term(&mut self) -> usize {
        let mut lhs = self.atom();
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.pos += 1;
                    let rhs = self.atom();
                    lhs = self.push(Node::Mul(lhs, rhs));
                }
                Some(Token::Slash) => {
                    self.pos += 1;
                    let rhs = self.atom();
                    lhs = self.push(Node::Div(lhs, rhs));
                }
                _ => break,
            }
        }
        lhs
    }

    fn atom(&mut self) -> usize {
        let t = self.peek();
        self.pos += 1;
        match t {
            Some(Token::Num(n)) => self.push(Node::Num(n)),
            Some(Token::Ident(name)) => self.push(Node::Var(name)),
            Some(Token::LParen) => {
                let e = self.expr();
                self.pos += 1;
                e
            }
            Some(Token::Max) => {
                self.pos += 1;
                let a = self.expr();
                self.pos += 1;
                let b = self.expr();
                self.pos += 1;
                self.push(Node::Max(a, b))
            }
            _ => self.push(Node::Num(0)),
        }
    }
}

fn parse_rule(src: &str) -> Rule {
    let mut p = RuleParser { tokens: tokenize(src), pos: 0, nodes: Vec::new() };
    let root = p.expr();
    Rule { nodes: p.nodes, root }
}

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    value: i64,
}

fn lookup(env: &Vec<Binding>, name: &str) -> i64 {
    for b in env {
        if b.name == name {
            return b.value;
        }
    }
    0
}

fn eval(rule: &Rule, node: usize, env: &Vec<Binding>) -> i64 {
    match &rule.nodes[node] {
        Node::Num(n) => *n,
        Node::Var(name) => lookup(env, name),
        Node::Add(a, b) => eval(rule, *a, env) + eval(rule, *b, env),
        Node::Sub(a, b) => eval(rule, *a, env) - eval(rule, *b, env),
        Node::Mul(a, b) => eval(rule, *a, env) * eval(rule, *b, env),
        Node::Div(a, b) => {
            let d = eval(rule, *b, env);
            if d == 0 { 0 } else { eval(rule, *a, env) / d }
        }
        Node::Max(a, b) => eval(rule, *a, env).max(eval(rule, *b, env)),
    }
}

fn price_order(inv: &Inventory, order: &Order, rule: &Rule) -> i64 {
    let mut subtotal = 0;
    let mut weight = 0;
    let mut qty = 0;
    for line in &order.lines {
        if let Some(p) = inv.product(line.sku) {
            subtotal += p.price_cents * line.qty;
            weight += p.weight_grams * line.qty;
            qty += line.qty;
        }
    }
    let discount = if qty >= 10 { subtotal / 20 } else { 0 };
    let env = vec![
        Binding { name: "subtotal".to_string(), value: subtotal },
        Binding { name: "weight".to_string(), value: weight },
        Binding { name: "qty".to_string(), value: qty },
        Binding { name: "discount".to_string(), value: discount },
    ];
    let shipping = eval(rule, rule.root, &env);
    let tax = (subtotal - discount) * 8 / 100;
    subtotal - discount + shipping + tax
}

// ---------- Routing: shortest paths between warehouses ----------

#[derive(Debug, Clone)]
struct Edge {
    to: usize,
    cost: i64,
}

#[derive(Debug)]
struct Graph {
    adj: Vec<Vec<Edge>>,
}

#[derive(Debug, Clone, Copy)]
struct HeapItem {
    cost: i64,
    node: usize,
}

/// A binary min-heap on a Vec.
struct MinHeap {
    items: Vec<HeapItem>,
}

impl MinHeap {
    fn push(&mut self, item: HeapItem) {
        self.items.push(item);
        let mut i = self.items.len() - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.items[parent].cost <= self.items[i].cost {
                break;
            }
            self.items.swap(parent, i);
            i = parent;
        }
    }

    fn pop(&mut self) -> Option<HeapItem> {
        if self.items.is_empty() {
            return None;
        }
        let last = self.items.len() - 1;
        self.items.swap(0, last);
        let top = self.items.pop();
        let mut i = 0;
        loop {
            let left = 2 * i + 1;
            let right = left + 1;
            let mut smallest = i;
            if left < self.items.len() && self.items[left].cost < self.items[smallest].cost {
                smallest = left;
            }
            if right < self.items.len() && self.items[right].cost < self.items[smallest].cost {
                smallest = right;
            }
            if smallest == i {
                break;
            }
            self.items.swap(i, smallest);
            i = smallest;
        }
        top
    }
}

fn build_graph(n: usize, rng: &mut Rng) -> Graph {
    let mut adj: Vec<Vec<Edge>> = Vec::new();
    for _ in 0..n {
        adj.push(Vec::new());
    }
    for i in 0..n {
        let next = (i + 1) % n;
        let cost = 5 + rng.below(20) as i64;
        adj[i].push(Edge { to: next, cost });
        adj[next].push(Edge { to: i, cost });
        let far = rng.below(n as u64) as usize;
        if far != i {
            let c = 10 + rng.below(40) as i64;
            adj[i].push(Edge { to: far, cost: c });
        }
    }
    Graph { adj }
}

fn shortest_paths(g: &Graph, from: usize) -> Vec<i64> {
    let mut dist = vec![1_000_000_000i64; g.adj.len()];
    let mut heap = MinHeap { items: Vec::new() };
    dist[from] = 0;
    heap.push(HeapItem { cost: 0, node: from });
    while let Some(item) = heap.pop() {
        if item.cost > dist[item.node] {
            continue;
        }
        for e in &g.adj[item.node] {
            let next = item.cost + e.cost;
            if next < dist[e.to] {
                dist[e.to] = next;
                heap.push(HeapItem { cost: next, node: e.to });
            }
        }
    }
    dist
}

// ---------- Reports ----------

fn pad_right(s: &str, width: usize) -> String {
    let mut out = s.to_string();
    while out.len() < width {
        out.push(' ');
    }
    out
}

fn pad_left(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut n = s.len();
    while n < width {
        out.push(' ');
        n += 1;
    }
    out.push_str(s);
    out
}

fn money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let c = cents.abs();
    let frac = c % 100;
    let pad = if frac < 10 { "0" } else { "" };
    format!("{}${}.{}{}", sign, c / 100, pad, frac)
}

fn category_report(inv: &Inventory) -> String {
    let cats = vec![Category::Hardware, Category::Electrical, Category::Plumbing, Category::Garden];
    let mut out = String::new();
    for c in &cats {
        let mut units = 0;
        let mut value = 0;
        let mut i = 0;
        while i < inv.products.len() {
            if inv.products[i].category == *c {
                units += inv.stock[i].on_hand;
                value += inv.stock[i].on_hand * inv.products[i].price_cents;
            }
            i += 1;
        }
        out.push_str(&pad_right(&category_name(*c), 12));
        out.push_str(&pad_left(&units.to_string(), 8));
        out.push_str(&pad_left(&money(value), 14));
        out.push('\n');
    }
    out
}

// ---------- Simulation ----------

#[derive(Debug)]
struct Stats {
    placed: u32,
    shipped: u32,
    cancelled: u32,
    rejected: u32,
    revenue_cents: i64,
    shipping_distance: i64,
}

fn seed_inventory(inv: &mut Inventory) {
    let names = vec!["bolt", "washer", "drill", "cable", "switch", "pipe", "valve", "hose", "rake", "shovel", "lamp", "fuse"];
    let mut sku = 100;
    for name in &names {
        let category = match sku % 4 {
            0 => Category::Hardware,
            1 => Category::Electrical,
            2 => Category::Plumbing,
            _ => Category::Garden,
        };
        let price = 99 + (sku as i64 * 37) % 4000;
        let weight = 50 + (sku as i64 * 91) % 3000;
        inv.add_product(sku, name, category, price, weight);
        sku += 1;
    }
}

fn random_order(id: u32, rng: &mut Rng, sites: usize) -> Order {
    let n_lines = 1 + rng.below(4);
    let mut lines = Vec::new();
    for _ in 0..n_lines {
        let sku = 100 + rng.below(13) as u32;
        let qty = 1 + rng.below(12) as i64;
        lines.push(OrderLine { sku, qty });
    }
    let customers = vec!["acme", "globex", "initech", "umbrella", "hooli"];
    let customer = customers[rng.below(5) as usize].to_string();
    Order { id, customer, lines, state: OrderState::Pending, total_cents: 0, destination: rng.below(sites as u64) as usize }
}

fn simulate(seed: u64, rounds: u32) -> Stats {
    let mut rng = Rng::new(seed);
    let mut inv = Inventory::new();
    seed_inventory(&mut inv);
    let graph = build_graph(16, &mut rng);
    let dist = shortest_paths(&graph, 0);
    let rule = parse_rule("max(500, weight / 100 * 3) + qty * 25 - discount / 10");
    let mut stats = Stats { placed: 0, shipped: 0, cancelled: 0, rejected: 0, revenue_cents: 0, shipping_distance: 0 };
    let mut open: Vec<Order> = Vec::new();
    for round in 0..rounds {
        let mut order = random_order(round, &mut rng, 16);
        stats.placed += 1;
        match reserve_order(&mut inv, &mut order) {
            Some(_) => stats.rejected += 1,
            None => {
                order.total_cents = price_order(&inv, &order, &rule);
                open.push(order);
            }
        }
        if round % 3 == 2 {
            let mut keep: Vec<Order> = Vec::new();
            while let Some(mut o) = open.pop() {
                if o.id % 7 == 3 {
                    if cancel_order(&mut inv, &mut o).is_none() {
                        stats.cancelled += 1;
                    }
                } else if transition(&mut o, OrderState::Paid).is_none() && ship_order(&mut inv, &mut o).is_none() {
                    stats.shipped += 1;
                    stats.revenue_cents += o.total_cents;
                    stats.shipping_distance += dist[o.destination];
                } else {
                    keep.push(o);
                }
            }
            open = keep;
        }
        if round % 10 == 9 {
            inv.restock();
        }
    }
    stats
}

fn self_test() {
    let rule = parse_rule("qty * 2 + max(3, weight) - 1");
    let env = vec![Binding { name: "qty".to_string(), value: 5 }, Binding { name: "weight".to_string(), value: 7 }];
    assert_eq!(eval(&rule, rule.root, &env), 16);
    assert_eq!(money(12345), "$123.45");
    assert_eq!(money(-7), "-$0.07");
    assert_eq!(tokenize("a+12"), vec![Token::Ident("a".to_string()), Token::Plus, Token::Num(12)]);
    let mut inv = Inventory::new();
    seed_inventory(&mut inv);
    assert_eq!(inv.reserve(999, 1), Some(StockError::UnknownSku(999)));
    assert!(inv.reserve(100, 5).is_none());
    assert_eq!(inv.available(100), 15);
    let mut order = Order { id: 1, customer: "t".to_string(), lines: Vec::new(), state: OrderState::Pending, total_cents: 0, destination: 0 };
    assert_eq!(reserve_order(&mut inv, &mut order), Some(OrderError::Empty));
    order.lines.push(OrderLine { sku: 101, qty: 2 });
    assert!(reserve_order(&mut inv, &mut order).is_none());
    assert_eq!(order.state, OrderState::Reserved);
    let e = transition(&mut order, OrderState::Shipped).unwrap();
    assert_eq!(describe_error(&e), "cannot Shipped from Reserved");
}

fn run_module() -> i64 {
    self_test();
    let stats = simulate(42, 300);
    let mut inv = Inventory::new();
    seed_inventory(&mut inv);
    println!("{:?}", stats);
    print!("{}", category_report(&inv));
    println!("inventory value {}", money(inv.total_value_cents()));
    stats.revenue_cents + stats.shipping_distance
}
