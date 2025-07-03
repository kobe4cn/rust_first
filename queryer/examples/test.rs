use std::{any, ops::Add, str::FromStr};

pub fn strtok<'a>(s: &mut &'a str, delimiter: char) -> &'a str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        // 由于 delimiter 可以是 utf8，所以我们需要获得其 utf8 长度，
        // 直接使用 len 返回的是字节长度，会有问题
        let suffix = &s[(i + delimiter.len_utf8())..];
        println!("prefix: {}, suffix: {}", prefix, suffix);
        *s = suffix;
        prefix
    } else {
        // 如果没找到，返回整个字符串，把原字符串指针 s 指向空串
        let prefix = *s;
        *s = "";
        prefix
    }
}

pub trait Parse {
    type Error;
    fn parse(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

// impl Parse for u8 {
//     fn parse(s: &str) -> Self {
//         let re = regex::Regex::new(r"^[0-9]+").unwrap();
//         if let Some(num) = re.captures(s) {
//             println!("num is: {:?}", num);
//             //如果map.get 0 为空 则返回0， 不然就把get 0的转换成str然后parse成u8，如果成功就范围，不成功就返回0
//             num.get(0).map_or(0, |m| m.as_str().parse().unwrap_or(0))
//         } else {
//             0
//         }
//     }
// }

// impl Parse for f64 {
//     fn parse(s: &str) -> Self {
//         let re = regex::Regex::new(r"^[0-9]+\.[0-9]+").unwrap();
//         if let Some(num) = re.captures(s) {
//             println!("num is: {:?}", num);
//             num.get(0)
//                 .map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0))
//         } else {
//             0.0
//         }
//     }
// }

// impl Parse for String {
//     fn parse(s: &str) -> Self {
//         let re = regex::Regex::new(r"[a-zA-Z]+").unwrap();
//         re.find_iter(s).map(|m| m.as_str().to_string()).collect()
//     }
// }

impl<T> Parse for T
where
    T: FromStr + Default,
    T::Err: ToString,
{
    type Error = String;
    fn parse(s: &str) -> Result<Self, Self::Error> {
        let re = match any::type_name::<T>() {
            "alloc::string::String" => regex::Regex::new(r"[a-zA-Z]+").unwrap(),
            _ => regex::Regex::new(r"[0-9]+(\.[0-9]+)?").unwrap(),
        };
        let result = re
            .find_iter(s)
            .map(|m| m.as_str().to_string())
            .collect::<Vec<String>>();
        result.concat().parse::<T>().map_err(|e| e.to_string())
        // if let So4me(num) = re.captures(s) {
        //     println!("num is: {:?}", num);
        //     num.get(0)
        //         .map_or(T::default(), |m| m.as_str().parse().unwrap_or(T::default()))
        // } else {
        //     T::default()
        // }
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Point {
    type Output = Point;
    fn add(self, rhs: &Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Add<f64> for &Point {
    type Output = Point;
    fn add(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x + rhs,
            y: self.y,
        }
    }
}

fn main() {
    // let s = u8::parse("123abc");
    // let b = u8::parse("1234abc");
    // println!("s is: {:?}", s);
    // //u8 是指从0开始到255结束的整数，b提取的数据为1234大于255
    // println!("b is: {:?}", b);

    // let c = f64::parse("123.456abc789");
    // println!("c is: {:?}", c);

    // let d = String::parse("123abcdef");
    // println!("d is: {:?}", d);

    // let e = String::parse("abc123def");
    // println!("e is: {:?}", e);

    // let f = String::parse("abc123.456def");
    // println!("f is: {:?}", f);

    // let g = String::parse("abc123.456def");
    // println!("g is: {:?}", g);

    // let h = String::parse("123.456");
    // println!("h is: {:?}", h);

    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = Point { x: 3.0, y: 4.0 };
    let p5 = &p1 + 1.0;
    let p4 = &p1 + &p2;
    let p3 = p1 + p2;
    println!("p3 is: {:?}", p3);

    println!("p4 is: {:?}", p4);
    println!("p5 is: {:?}", p5);
}
