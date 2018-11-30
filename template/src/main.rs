use std::{
    io::{self, Read},
};

fn get_input() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() -> Result<(), failure::Error> {
    let input = get_input()?;

    // solve problem

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(1, 1)
    }
}
