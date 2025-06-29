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
