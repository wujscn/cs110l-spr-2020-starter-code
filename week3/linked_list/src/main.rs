use linked_list::LinkedList;

use crate::linked_list::ComputeNorm;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<u32> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    // If you implement iterator trait:
    println!("======== test iter for loop ========");
    for val in &list {
       print!("{} ", val);
    }
    println!();
    println!("{}", list);    

    let mut str_list: LinkedList<String> = LinkedList::new();
    for i in 1..12 {
        str_list.push_front(i.to_string());
    }
    println!("string list:{}", str_list);
    println!("======== test clone ========");
    let str_list_clone = str_list.clone();
    let str_list_eq = str_list;
    println!("string list clone:{}", str_list_clone);
    println!("string list eq:{}", str_list_eq);
    println!("test \"str_list_eq == str_list_clone\": {}", str_list_eq == str_list_clone);

    println!("======== test computenorm ========");
    let mut f64_list: LinkedList<f64> = LinkedList::new();
    let init_vec = vec![5.0, 12.0];
    for i in init_vec {
        f64_list.push_front(i);
    }
    println!("{} => {}", f64_list, f64_list.compute_norm());
}
