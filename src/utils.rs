pub fn prepend<T>(e: T, mut es: Vec<T>) -> Vec<T> {
    es.insert(0, e);
    es
}

pub fn cat(mut a: String, b: String) -> String {
    a.push_str(b.as_str());
    a
}

pub fn chop(mut s: String) -> String {
    s.remove(0);
    s
}

pub fn chopchop(mut s: String) -> String {
    s.pop();
    s.remove(0);
    s
}
