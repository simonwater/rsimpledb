use rspression::{DefaultEnvironment, Environment, RspRunner, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("----evaluate----");
    eval()?;
    println!("----calculate----");
    calc()
}

fn eval() -> Result<(), Box<dyn std::error::Error>> {
    // Basic arithmetic
    let mut runner = RspRunner::new();

    // Simple expression
    println!("1 + 2 * 3 = {}", runner.execute("1 + 2 * 3")?);

    // With variables
    let mut env = DefaultEnvironment::new();
    env.put("a".to_string(), Value::Integer(1));
    env.put("b".to_string(), Value::Integer(2));
    env.put("c".to_string(), Value::Integer(3));
    println!(
        "a + b * c = {}",
        runner.execute_with_env("a + b * c", &mut env)?
    );
    println!("{}", runner.execute_with_env("a + b * c >= 6", &mut env)?);

    Ok(())
}

fn calc() -> Result<(), Box<dyn std::error::Error>> {
    let mut srcs = Vec::new();
    srcs.push("x = a + b * c");
    srcs.push("a = m + n");
    srcs.push("b = a * 2");
    srcs.push("c = n + w + b");

    let mut runner = RspRunner::new();
    let mut env = DefaultEnvironment::new();
    env.put("m".to_string(), Value::Integer(2));
    env.put("n".to_string(), Value::Integer(4));
    env.put("w".to_string(), Value::Integer(6));

    runner.execute_multiple_with_env(&srcs, &mut env).unwrap();
    println!("x = {}", env.get("x").unwrap().as_integer());
    println!("a = {}", env.get("a").unwrap().as_integer());
    println!("b = {}", env.get("b").unwrap().as_integer());
    println!("c = {}", env.get("c").unwrap().as_integer());

    Ok(())
}
