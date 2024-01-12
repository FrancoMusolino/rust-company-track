use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use domain::Repository;
use serde_json::json;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{stdin, Write},
    path::Path,
    rc::Rc,
};

pub mod company;
pub mod domain;
use crate::company::Employee;
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

type Database = rusqlite::Connection;

pub fn run() -> Result<(), Box<dyn Error>> {
    let path = if env::var("PRODUCTION").is_ok() {
        Path::new("./db/company.prod.db")
    } else {
        Path::new("./db/company.dev.db")
    };

    let db = rusqlite::Connection::open(path)?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS departments (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE
    )",
        [],
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS employees (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            department_id TEXT NOT NULL REFERENCES departments(id)
    )",
        [],
    )?;

    let mut company = Company::get(&db)?;
    let mut next = next_choice().unwrap();

    while next != Possibilities::Quit {
        match next {
            Possibilities::AddEntry => add_new_entry(&db, &mut company)?,
            Possibilities::ViewList => view_list(&company)?,
            Possibilities::GenerateReport => generate_report(&company)?,
            _ => (),
        }

        next = next_choice().unwrap();
    }

    Ok(())
}

fn add_new_entry(db: &Database, company: &mut Company) -> Result<(), Box<dyn Error>> {
    let employee = ask_for_employee()?;
    let department = ask_for_department(
        &company
            .departments
            .iter()
            .map(|department| department.name.clone())
            .collect::<Vec<String>>(),
    )?;

    company.add_entry(department, employee)?;
    company.save(db)?;

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

fn view_list(company: &Company) -> Result<(), Box<dyn Error>> {
    for department in company.departments.iter() {
        println!(
            "\n{}",
            format!("Department {}", department.name).bold().underline()
        );

        let employees_in_department = company
            .employees
            .iter()
            .cloned()
            .filter(|employee| employee.department_id == department.id)
            .collect::<Vec<Rc<Employee>>>();

        for (i, employee) in employees_in_department.iter().enumerate() {
            println!("{}. {}", i + 1, employee.name);
        }

        println!();
    }

    Ok(())
}

fn generate_report(company: &Company) -> Result<(), Box<dyn Error>> {
    let path = Path::new("report.json");
    let mut file = File::create(path)?;

    let total_employees = company.get_total_employees();
    let mut distribution: HashMap<&String, String> = HashMap::new();

    for department in company.departments.iter() {
        let employees = company.get_employees_by_department(&department.id);

        distribution.insert(
            &department.name,
            format!(
                "{:.2$}% ({} employees)",
                ((employees as f32) * 100.0) / (total_employees as f32),
                employees,
                2,
            ),
        );
    }

    let report = json!({
        "departments": company.departments.len(),
        "employees": total_employees,
        "company_distribution": distribution
    });

    file.write(&report.to_string().as_bytes())?;

    Ok(())
}
