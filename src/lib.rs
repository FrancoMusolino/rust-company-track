use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use domain::{AggregateRoot, Repository};
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
    AddDepartment,
    HireEmployee,
    ViewList,
    GenerateReport,
    Quit,
}

fn next_choice() -> Option<Possibilities> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Quieres...")
        .item("Añadir un nuevo departamento")
        .item("Contratar empleado")
        .item("Ver lista")
        .item("Generar reporte")
        .item("Salir")
        .interact()
        .unwrap();

    match selection {
        0 => Some(Possibilities::AddDepartment),
        1 => Some(Possibilities::HireEmployee),
        2 => Some(Possibilities::ViewList),
        3 => Some(Possibilities::GenerateReport),
        4 => Some(Possibilities::Quit),
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
            Possibilities::AddDepartment => add_department(&db, &mut company)?,
            Possibilities::HireEmployee => hire_employee(&db, &mut company)?,
            Possibilities::ViewList => view_list(&company)?,
            Possibilities::GenerateReport => generate_report(&company)?,
            _ => (),
        }

        next = next_choice().unwrap();
    }

    Ok(())
}

fn add_department(db: &Database, company: &mut Company) -> Result<(), Box<dyn Error>> {
    let department = ask_for_stdin("Ingrese el nuevo departamento")?;

    if let Err(err) = company.add_department(department) {
        eprintln!("{err}");
    } else {
        company.save(db)?;
        company.commit();
    };

    Ok(())
}

fn hire_employee(db: &Database, company: &mut Company) -> Result<(), Box<dyn Error>> {
    if company.departments.len() == 0 {
        eprintln!("Primero debe añadir un departamento");
        return Ok(());
    }

    let employee = ask_for_stdin("Ingrese el nombre del empleado")?;
    let department = ask_for_department(
        &company
            .departments
            .iter()
            .map(|department| department.name.clone())
            .collect::<Vec<String>>(),
    )?;

    if let Err(err) = company.hire_employee(employee, department) {
        eprintln!("{err}");
    } else {
        company.save(db)?;
        company.commit();
    };

    Ok(())
}

fn ask_for_stdin(label: &str) -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    println!("{label}");
    stdin().read_line(&mut input)?;

    Ok(input)
}

fn ask_for_department(departments: &[String]) -> Result<String, Box<dyn Error>> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Seleccione un departamento")
        .items(departments)
        .interact()?;

    let department = departments.get(selection).unwrap().to_owned();

    Ok(department)
}

fn view_list(company: &Company) -> Result<(), Box<dyn Error>> {
    for department in company.departments.iter() {
        println!(
            "\n{}",
            format!("Departamento {}", department.name)
                .bold()
                .underline()
        );

        let employees_in_department = company
            .employees
            .iter()
            .cloned()
            .filter(|employee| employee.department_id == department.id)
            .collect::<Vec<Rc<Employee>>>();

        if employees_in_department.len() == 0 {
            println!("Sin empleados");
        }

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
