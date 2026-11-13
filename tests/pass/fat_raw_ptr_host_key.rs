// The ts-rust ls_program.rs host registry shape: hosts keyed by the data
// address of an Rc<dyn Trait>.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

trait CompilerHost {
    fn name(&self) -> String;
    fn release(&self) {}
}
struct Host(String);
impl CompilerHost for Host {
    fn name(&self) -> String { self.0.clone() }
}

thread_local! {
    static HOST_USERS: RefCell<HashMap<usize, u32>> = RefCell::new(HashMap::new());
}

fn host_key(host: &Rc<dyn CompilerHost>) -> usize {
    Rc::as_ptr(host).cast::<()>().addr()
}

fn hold(hosts: &[Rc<dyn CompilerHost>]) {
    HOST_USERS.with(|users| {
        let mut users = users.borrow_mut();
        for host in hosts {
            *users.entry(host_key(host)).or_insert(0) += 1;
        }
    });
}

fn release(hosts: &[Rc<dyn CompilerHost>]) -> Vec<String> {
    let unused: Vec<Rc<dyn CompilerHost>> = HOST_USERS.with(|users| {
        let mut users = users.borrow_mut();
        hosts
            .iter()
            .filter(|host| {
                let key = host_key(host);
                let count = users.get_mut(&key).expect("held");
                *count -= 1;
                if *count == 0 {
                    users.remove(&key);
                    return true;
                }
                false
            })
            .cloned()
            .collect()
    });
    unused.iter().map(|h| { h.release(); h.name() }).collect()
}

fn main() {
    let a: Rc<dyn CompilerHost> = Rc::new(Host("a".into()));
    let b: Rc<dyn CompilerHost> = Rc::new(Host("b".into()));
    hold(&[a.clone(), b.clone()]);
    hold(&[a.clone()]);
    println!("{:?}", release(&[a.clone(), b.clone()]));
    println!("{:?}", release(&[a.clone()]));
    println!("{}", HOST_USERS.with(|u| u.borrow().len()));
}
