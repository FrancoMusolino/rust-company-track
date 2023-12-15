use dialoguer::{theme::ColorfulTheme, Select};
use rusqlite::Connection;
use std::{error::Error, io::stdin};

pub mod company;
pub use company::Company;

#[derive(PartialEq)]
enum Possibilities {
    AddEntry,
    ViewList,
    GenerateReport,
    Quit,
}

fn next_choice() -> Option<Possibilities> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want...")
        .item("Add new entry")
        .item("View list")
        .item("Generate report")
        .item("Quit")
        .interact()
        .unwrap();

    match selection {
        0 => Some(Possibilities::AddEntry),
        1 => Some(Possibilities::ViewList),
        2 => Some(Possibilities::GenerateReport),
        3 => Some(Possibilities::Quit),
        _ => None,
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open("./db/company.db")?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS departments (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE
    )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS employees (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        department_id TEXT NOT NULL REFERENCES departments(id)
    )",
        [],
    )?;

    let mut next = next_choice().unwrap();

    while next != Possibilities::Quit {
        match next {
            Possibilities::AddEntry => add_new_entry(&connection)?,
            Possibilities::ViewList => view_list(&connection)?,
            _ => (),
        }

        next = next_choice().unwrap();
    }

    Ok(())
}

fn add_new_entry(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let mut company = Company::build_from_existing(&connection)?;

    let employee = ask_for_employee()?;
    let department = ask_for_department(&company.departments)?;

    company.add_entry(department, employee, &connection)?;

    Ok(())
}

fn ask_for_employee() -> Result<String, Box<dyn Error>> {
    let mut employee = String::new();
    println!("Enter the employee's name");
    stdin().read_line(&mut employee)?;

    Ok(employee)
}

fn ask_for_department(departments: &[String]) -> Result<String, Box<dyn Error>> {
    let mut department = String::new();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose an existing department or create one")
        .items(departments)
        .item("New")
        .interact()?;

    if selection + 1 > departments.len() {
        println!("Enter the department's name");
        stdin().read_line(&mut department)?;
    } else {
        department = departments.get(selection).unwrap().to_owned();
    }

    Ok(department)
}

fn view_list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let company = Company::build_from_existing(&connection)?;

    company.view_list();

    Ok(())
}
