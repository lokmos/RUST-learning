# 所有权
## 所有权
### 所有权和 move 语义

Rust 给出了如下规则：

- 一个值只能被一个变量所拥有，这个变量被称为所有者（Each value in Rust has a variable that’s called its owner）。
- 一个值同一时刻只能有一个所有者（There can only be one owner at a time），也就是说不能有两个变量拥有相同的值。变量赋值、参数传递、函数返回等行为，旧的所有者会把值的所有权转移给新的所有者，以便保证单一所有者的约束。
- 当所有者离开作用域，其拥有的值被丢弃（When the owner goes out of scope, the value will be dropped），内存得到释放。

所有权解决了堆上数据的多重引用问题，但同时也会使的简单数据的使用变得复杂，rust 提供了两种解决方案

1. 如果不希望值的所有权被转移，在 Move 语义外，Rust 提供了 **Copy 语义**。如果一个数据结构实现了 `Copy trait`，那么它就会使用 Copy 语义。这样，在你赋值或者传参时，值会自动**按位拷贝（浅拷贝）**。
2. 如果不希望值的所有权被转移，又无法使用 Copy 语义，可以 **借用** 数据，这点会在之后的内容讨论。

### Copy 语义 & Copy Trait

当要移动一个值，如果值的类型实现了 Copy trait，就会自动使用 Copy 语义进行拷贝，否则使用 Move 语义进行移动

实现了 Copy 的类型

- 原生类型，包括函数、不可变引用和裸指针实现了 Copy；
- 数组和元组，如果其内部的数据结构实现了 Copy，那么它们也实现了 Copy；
- 可变引用没有实现 Copy；
- 非固定大小的数据结构，没有实现 Copy。

## 值的借用

### Borrow 语义

Borrow 语义允许一个值的所有权，在不发生转移的情况下，被其它上下文使用

- Borrow 语义通过引用语法 **（& 或者 &mut）** 来实现
- 在 Rust 下，所有的引用都只是借用了“临时使用权”，它并不破坏值的单一所有权约束
- 在默认情况下，Rust 的借用都是只读的

### 只读借用/引用

本质上，引用是一个受控的指针，指向某个特定的类型。

在 Rust 中，没有传引用的概念。**Rust 所有的参数传递都是传值**，不管是 Copy 还是 Move。所以在 Rust 中，你必须显式地把某个数据的引用，传给另一个函数
- Rust 的引用实现了 Copy trait，所以按照 Copy 语义，这个引用会被复制一份交给要调用的函数。
- 对这个函数来说，它并不拥有数据本身，数据只是临时借给它使用，所有权还在原来的拥有者那里

在 Rust 里，引用是一等公民，**和其他数据类型地位相等**

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
    let data1 = &data;

    println!(
        "addr of data' value: {:p}({:p}), addr of &data: {:p}, addr of data1: {:p}",
        &data, data1, &&data, &data1
    );
    sum(&data);
}

fn sum(data: &Vec<u32>) -> u32 {
    println!(
        "addr of data's value: {:p}, addr of data: {:p}",
        data, &data
    );
    data.iter().sum()
}
```
- `data1`、`&data` 和传到 `sum()` 里的 `data1’` 都指向 `data` 本身，这个值的地址是固定的
- 但是它们引用的地址都是不同的
- **只读引用实现了 Copy trait，也就意味着引用的赋值、传参都会产生新的浅拷贝**

### 借用的生命周期及其约束

**借用不能超过（outlive）值的生存期**

堆变量的生命周期不具备任意长短的灵活性，因为堆上内存的生死存亡，跟栈上的所有者牢牢绑定

### 可变借用/引用

为了保证内存安全，Rust 对可变引用的使用也做了严格的约束：
- **在一个作用域内，仅允许一个活跃的可变引用**。所谓活跃，就是真正被使用来修改数据的可变引用，如果只是定义了，却没有使用或者当作只读引用使用，不算活跃。
- 在一个作用域内，活跃的可变引用（写）和只读引用（读）是互斥的，不能同时存在。

## 智能指针

### Rc

对某个数据结构 `T`，我们可以创建引用计数 Rc，使其有多个所有者。Rc 会把对应的数据结构创建在堆上
- 需要注意，Rc 是**只读**的

对一个 Rc 结构进行 `clone()`，不会将其内部的数据复制，只会增加引用计数。而当一个 Rc 结构离开作用域被 `drop()` 时，也只会减少其引用计数，直到引用计数为零，才会真正清除对应的内存。

```rust
use std::rc::Rc;
fn main() {
    let a = Rc::new(1);
    let b = a.clone();
    let c = a.clone();
}
```
- 上面的代码我们创建了三个 Rc，分别是 a、b 和 c。它们共同指向堆上相同的数据，也就是说，堆上的数据有了三个共享的所有者。在这段代码结束时，c 先 drop，引用计数变成 2，然后 b drop、a drop，引用计数归零，堆上内存被释放。
- 编译器的角度，abc 都各自拥有一个 Rc，所以这并不会违反单一所有权

** `Rc::clone()` ** 的实现
```rust
fn clone(&self) -> Rc<T> {
    // 增加引用计数
    self.inner().inc_strong();
    // 通过 self.ptr 生成一个新的 Rc 结构
    Self::from_inner(self.ptr)
}
```

### `Box::leak()` 机制

在所有权模型下，堆内存的生命周期，和创建它的栈内存的生命周期保持一致。所以 Rc 的实现似乎与此格格不入。如果完全按照单一所有权模型，Rust 是无法处理 Rc 这样的引用计数的。

Rust 必须提供一种机制，让代码可以像 C/C++ 那样，创建不受栈内存控制的堆内存，从而绕过编译时的所有权规则。Rust 提供的方式是 Box::leak()
- Box 是 Rust 下的智能指针，它可以强制把任何数据结构创建在堆上，然后在栈上放一个指针指向这个数据结构，但此时堆内存的生命周期仍然是受控的，跟栈上的指针一致
- `Box::leak()`，顾名思义，它创建的对象，从堆内存上泄漏出去，不受栈内存控制，是一个自由的、生命周期可以大到和整个进程的生命周期一致的对象。

有了 `Box::leak()`，我们就可以跳出 Rust 编译器的静态检查，保证 Rc 指向的堆内存，有最大的生命周期，然后我们再通过引用计数，在合适的时机，结束这段内存的生命周期


### Rust 所有权的静态检查和动态检查

静态检查，靠编译器保证代码符合所有权规则

动态检查，通过 `Box::leak` 让堆内存拥有不受限的生命周期，然后在运行过程中，通过对引用计数的检查，保证这样的堆内存最终会得到释放

### 内部可变性

#### 外部可变性

当我们用 `let mut` 显式地声明一个可变的值，或者，用 `&mut` 声明一个可变引用时，编译器可以在编译时进行严格地检查，保证只有可变的值或者可变的引用，才能修改值内部的数据，这被称作外部可变性（exterior mutability），外部可变性通过 `mut` 关键字声明。

#### RefCell

绕开这个编译时的检查，对并未声明成 `mut` 的值或者引用，也进行修改
- 编译器的眼里，值是只读的，但是在运行时，这个值可以得到可变借用，从而修改内部的数据
- 通过使用 RefCell 的 `borrow_mut()` 方法，获得一个可变的内部引用
- 通过 RefCell 的  `borrow()` 方法，获得一个不可变的内部引用

#### 使用 Rc 和 RefCell 实现 DAG
```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    id: usize,
    downstream: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            downstream: None,
        }
    }

    pub fn update_downstream(&mut self, downstream: Rc<RefCell<Node>>) {
        self.downstream = Some(downstream);
    }

    pub fn get_downstream(&self) -> Option<Rc<RefCell<Node>>> {
        self.downstream.as_ref()
                        .map(|v| v.clone())
    }
}

fn main() {
    let mut node1 = Node::new(1);
    let mut node2 = Node::new(2);
    let mut node3 = Node::new(3);
    let node4 = Node::new(4);
    node3.update_downstream(Rc::new(RefCell::new(node4)));
    node1.update_downstream(Rc::new(RefCell::new(node3)));
    node2.update_downstream(node1.get_downstream().unwrap());

    println!("node1: {:?}, node2: {:?}", node1, node2);

    let node5 = Node::new(5);
    let node3 = node1.get_downstream().unwrap();
    node3.borrow_mut().downstream = Some(Rc::new(RefCell::new(node5)));

    println!("node1: {:?}, node2: {:?}", node1, node2);
}
```

### Arc 和 Mutex/RwLock

Rc 为了性能，使用的不是线程安全的引用计数器

另一个引用计数的智能指针：Arc，它实现了线程安全的引用计数器
- Arc 内部的引用计数使用了 Atomic Usize ，而非普通的 usize。Atomic Usize 是 usize 的原子类型，它使用了 CPU 的特殊指令，来保证多线程下的安全

Rust 实现两套不同的引用计数数据结构，完全是为了性能考虑。如果不用跨线程访问，可以用效率非常高的 Rc；如果要跨线程访问，那么必须用 Arc

同样的，RefCell 也不是线程安全的，如果我们要在多线程中，使用内部可变性，Rust 提供了 Mutex 和 RwLock。
- Mutex 是互斥量，获得互斥量的线程对数据独占访问，RwLock 是读写锁，获得写锁的线程对数据独占访问，但当没有写锁的时候，允许有多个读锁
- Mutex 和 RwLock 都用在多线程环境下，对共享数据访问的保护上。刚才中我们构建的 DAG 如果要用在多线程环境下，需要把 `Rc<RefCell<T>>` 替换为 `Arc<Mutex<T>>` 或者 `Arc<RwLock<T>>`

| 访问方式       | 数据                 | 不可变借用              | 可变借用                |
|----------------|----------------------|--------------------------|--------------------------|
| 单一所有权     | T                    | &T                       | &mut T                  |
|                | Rc<T>                | &Rc<T>                   | 无法得到可变借用        |
| 单线程         | Rc<RefCell<T>>       | v.borrow()               | v.borrow_mut()          |
|                | Arc<T>               | &Arc<T>                  | 无法得到可变借用        |
| 共享所有权     | Arc<Mutex<T>>        | v.lock()                 | v.lock()                 |
| 多线程         | Arc<RwLock<T>>       | v.read()                 | v.write()                |

# 生命周期

在 Rust 中，除非显式地做 Box::leak() / Box::into_raw() / ManualDrop 等动作，一般来说，堆内存的生命周期，会默认和其栈内存的生命周期绑定在一起。

## 值的生命周期

**静态生命周期**
- 如果一个值的生命周期贯穿**整个进程的生命周期**，那么我们就称这种生命周期为静态生命周期。
- 当值拥有静态生命周期，其引用也具有静态生命周期。我们在表述这种引用的时候，可以用 `'static` 来表示。比如： `&'static str`  代表这是一个具有静态生命周期的字符串引用。
- 一般来说，全局变量、静态变量、字符串字面量（string literal）等，都拥有静态生命周期
  - 堆内存，如果使用了 `Box::leak` 后，也具有静态生命周期

**动态生命周期**
- 如果一个值是在某个作用域中定义的，也就是说它被创建在栈上或者堆上，那么其生命周期是动态的
- 当这个值的作用域结束时，值的生命周期也随之结束
- 对于动态生命周期，我们约定用 `'a` 、`'b` 或者 `'hello` 这样的小写字符或者字符串来表述

**总结**
- 分配在堆和栈上的内存有其各自的作用域，它们的生命周期是动态的。
- 全局变量、静态变量、字符串字面量、代码等内容，在编译时，会被编译到可执行文件中的 BSS/Data/RoData/Text 段，然后在加载时，装入内存。因而，它们的生命周期和进程的生命周期一致，所以是静态的。
- 所以，函数指针的生命周期也是静态的，因为函数在 Text 段中，只要进程活着，其内存一直存在。

## 编译器如何识别生命周期

```rust
fn main() {
    let s1 = String::from("Lindsey");
    let s2 = String::from("Rosie");

    let result = max(&s1, &s2);
    println!("The longer string is {}", result);
}

fn max(s1: &str, s2: &str) -> &str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}
```

这段代码是无法编译通过的，它会报错 “missing lifetime specifier” ，也就是说，编译器在编译 `max()` 函数时，无法判断 `s1`、`s2` 和返回值的生命周期。

当出现了多个参数，它们的生命周期可能不一致时，返回值的生命周期就不好确定了。编译器在编译某个函数时，并不知道这个函数将来有谁调用、怎么调用，所以，函数本身携带的信息，就是编译器在编译时使用的全部信息。

```rust
fn main() {
    let s1 = String::from("Lindsey");
    let s2 = String::from("Rosie");

    let result = max(&s1, &s2);
    println!("The longer string is {}", result);
}

fn max<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}
```
- 需要我们在函数签名中提供生命周期的信息，也就是生命周期标注
- 生命周期参数的描述方式和泛型参数一致，不过只使用小写字母。这里，两个入参 s1、 s2，以及返回值都用 `'a` 来约束。生命周期参数，描述的是参数和参数之间、参数和返回值之间的关系，并不改变原有的生命周期。

编译器会通过一些简单的规则为函数自动添加标注：
- 所有引用类型的参数都有独立的生命周期  `'a` 、`'b` 等。
- 如果只有一个引用型输入，它的生命周期会赋给所有输出。
- 如果有多个引用类型的参数，其中一个是 self，那么它的生命周期会赋给所有输出。

生命周期标注的目的是，**在参数和返回值之间建立联系或者约束**。调用函数时，传入的参数的生命周期需要大于等于（outlive）标注的生命周期。
- 当每个函数都添加好生命周期标注后，编译器，就可以从函数调用的上下文中分析出，在传参时，引用的生命周期，是否和函数签名中要求的生命周期匹配。如果不匹配，就违背了“引用的生命周期不能超出值的生命周期”，编译器就会报错。
- 数据结构的生命周期标注也是类似，使用数据结构时，数据结构自身的生命周期，需要小于等于其内部字段的所有引用的生命周期。
  - 比如下面的例子，`Employee` 的 `name` 和 `title` 是两个字符串引用，`Employee` 的生命周期不能大于它们，否则会访问失效的内存
  - ```rust
        struct Employee<'a, 'b> {
        name: &'a str,
        title: &'b str,
        age: u8,
        }
    ```

## 需要审慎考虑返回值的生命周期

```rust
pub fn strtok(s: &mut &str, delimiter: char) -> &str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        // 由于 delimiter 可以是 utf8，所以我们需要获得其 utf8 长度，
        // 直接使用 len 返回的是字节长度，会有问题
        let suffix = &s[(i + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else { // 如果没找到，返回整个字符串，把原字符串指针 s 指向空串
        let prefix = *s;
        *s = "";
        prefix
    }
}
fn main() {
    let s = "hello world".to_owned();
    let mut s1 = s.as_str();
    let hello = strtok(&mut s1, ' ');
    println!("hello is: {}, s1: {}, s: {}", hello, s1, s);
}
```
- 当我们尝试运行这段代码时，会遇到生命周期相关的编译错误
- 按照编译器的规则， `&mut &str` 添加生命周期后变成  `&'b mut &'a str`，这将导致返回的  `'&str` 无法选择一个合适的生命周期

返回值和字符串引用 `&str` 本身有关
```rust
pub fn strtok<'a>(s: &'a mut &str, delimiter: char) -> &'a str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        let suffix = &s[(i + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}
fn main() {
    let s = "hello world".to_owned();
    let mut s1 = s.as_str();
    let hello = strtok(&mut s1, ' ');
    println!("hello is: {}, s1: {}, s: {}", hello, s1, s);
}
```

如果返回值的生命周期写成了与指向字符串的可变引用一致
```rust
pub fn strtok<'a>(s: &'a mut &str, delimiter: char) -> &'a str {...}
```
- 编译器会报错，`s1` 同时存在可变引用和不可变引用
- `s1`的可变引用的周期和返回值绑定了.在 `hello` 使用结束前,编译器认为 `s1` 的可变引用一直存在.
  - 现在我们把对字符串的可变引用的生命周期和返回值的生命周期关联起来了，`hello` 得到了 `strtok` 的返回值，所以从编译器看来，`&mut s1` 的生命周期并没有结束，所以发生了 `&mut` 和 `&` 同时存在的局面。

# 内存管理

## 值的创建

理论上，编译时可以确定大小的值都会放在栈上，包括 Rust 提供的原生类型比如字符、数组、元组（tuple）等，以及开发者自定义的固定大小的结构体（struct）、枚举（enum） 等。

如果数据结构的大小无法确定，或者它的大小确定但是在使用时需要更长的生命周期，就最好放在堆上。

### struct & tuple

Rust 在内存中排布数据时，**会根据每个域的对齐（aligment）对数据进行重排**，使其内存大小和访问效率最好。
- 可以使用 `#[repr]` 宏，强制让 Rust 编译器不做优化，和 C 的行为一致，这样，Rust 代码可以方便地和 C 代码无缝交互

### enum

在 Rust 下它是一个标签联合体（tagged union），它的大小是**标签的大小，加上最大类型的长度**
- Option 是有值 / 无值这种最简单的枚举类型，Result 包括成功返回数据和错误返回数据的枚举类型
- tag 后的内存，会根据其对齐大小进行对齐
- 一般而言，64 位 CPU 下，enum 的最大长度是：最大类型的长度 + 8，因为 64 位 CPU 的最大对齐是 64bit，也就是 8 个字节

Rust 编译器会对 enum 做一些额外的优化，让某些常用结构的内存布局更紧凑
- Option 配合带有引用类型的数据结构，比如 &u8、Box、Vec、HashMap ，没有额外占用空间
  - 对于 Option 结构而言，它的 tag 只有两种情况：0 或 1， tag 为 0 时，表示 None，tag 为 1 时，表示 Some
  - 正常来说，当我们把它和一个引用放在一起时，虽然 tag 只占 1 个 bit，但 64 位 CPU 下，引用结构的对齐是 8，所以它自己加上额外的 padding，会占据 8 个字节，一共 16 字节，这非常浪费内存
  - Rust 是这么处理的，我们知道，引用类型的第一个域是个指针，而指针是不可能等于 0 的，但是我们可以复用这个指针：当其为 0 时，表示 None，否则是 Some

### Vec<T> & String

`String` 结构的内部就是一个 `Vec<u8>`

`Vec<T> `结构是 3 个 word 的胖指针，包含：一个指向堆内存的指针 pointer、分配的堆内存的容量 capacity，以及数据在堆内存的长度 length
- 很多动态大小的数据结构，在创建时都有类似的内存布局：栈内存放的胖指针，指向堆内存分配出来的数据，Rc 也是如此

## 值的使用

对 Rust 而言，一个值如果没有实现 Copy，在赋值、传参以及函数返回时会被 Move
- 其实 Copy 和 Move 在内部实现上，**都是浅层的按位做内存复制**，只不过 Copy 允许你访问之前的变量，而 Move 不允许

内存复制是个很重的操作，效率很低
- 但是，如果你要复制的只是原生类型（Copy）或者栈上的胖指针（Move），不涉及堆内存的复制也就是深拷贝（deep copy），那这个效率是非常高的，我们不必担心每次赋值或者每次传参带来的性能损失
- 也有一个例外，要说明：对栈上的大数组传参，由于需要复制整个数组，会影响效率。所以，一般我们建议在栈上不要放大数组，如果实在需要，那么传递这个数组时，最好用传引用而不是传值

在使用值的过程中，除了 Move，还需要注意值的动态增长。因为 Rust 下，集合类型的数据结构，都会在使用过程中自动扩展
- 以一个 `Vec<T>` 为例，当你使用完堆内存目前的容量后，还继续添加新的内容，就会触发堆内存的自动增长。有时候，集合类型里的数据不断进进出出，导致集合一直增长，但只使用了很小部分的容量，内存的使用效率很低，所以你要考虑使用，比如 `shrink_to_fit` 方法，来节约对内存的使用

## 值的销毁

`Drop trait` 类似面向对象编程中的析构函数，当一个值要被释放，它的 `Drop trait` 会被调用

比如变量 `greeting` 是一个字符串，在退出作用域时，其 `drop()` 函数被自动调用，释放堆上包含 `“hello world”` 的内存，然后再释放栈上的内存

如果要释放的值是一个复杂的数据结构，比如一个结构体，那么这个结构体在调用 `drop()` 时，会依次调用每一个域的 `drop()` 函数，如果域又是一个复杂的结构或者集合类型，就会递归下去，直到每一个域都释放干净。

## 堆内存释放

所有权机制规定了，一个值只能有一个所有者，所以在释放堆内存的时候，整个过程简单清晰，就是单纯调用 `Drop trait`，不需要有其他顾虑

如果你定义的 `drop()` 函数和系统自定义的 `drop()` 函数都 `drop()` 某个域，Rust 编译器会确保，这个域只会被 drop 一次

## 释放其他资源

Rust 的 `Drop trait` 主要是为了应对堆内存释放的问题，其实，它还可以释放任何资源，比如 socket、文件、锁等等。Rust 对所有的资源都有很好的 RAII 支持