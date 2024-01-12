use std::{error::Error, rc::Rc};

use crate::{
    domain::{AggregateRoot, DomainEvent, Repository},
    Database,
};
use cuid;

#[derive(Debug)]
pub enum CompanyEvents {
    DepartmentAdded(Rc<Department>),
    EmployeeHired(Rc<Employee>),
}

#[derive(Debug)]
pub struct Department {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Employee {
    id: String,
    pub name: String,
    pub department_id: String,
}

#[derive(Default, Debug)]
pub struct Company {
    pub departments: Vec<Rc<Department>>,
    pub employees: Vec<Rc<Employee>>,
    events: Vec<DomainEvent<CompanyEvents>>,
}

impl AggregateRoot<CompanyEvents> for Company {
    fn apply(&mut self, event: CompanyEvents) -> () {
        self.events.push(DomainEvent { event })
    }

    fn commit(&mut self) -> () {
        self.events = Vec::new()
    }

    fn get_uncommited_events(&self) -> &Vec<DomainEvent<CompanyEvents>> {
        &self.events
    }
}

impl Repository<CompanyEvents, Company> for Company {
    fn get(db: &Database) -> Result<Self, Box<dyn Error>> {
        let mut company = Company::default();
        let mut departments_stmt = db.prepare("SELECT * FROM departments")?;
        let mut employees_stmt = db.prepare("SELECT * FROM employees WHERE department_id = $1")?;

        let departments = departments_stmt.query_map([], |row| {
            Ok(Rc::new(Department {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
            }))
        })?;

        for department in departments {
            let dpt = department.unwrap();

            let employees = employees_stmt.query_map([&dpt.id], |row| {
                Ok(Rc::new(Employee {
                    id: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                    department_id: row.get(2).unwrap(),
                }))
            })?;

            for employee in employees {
                company.employees.push(employee.unwrap().clone());
            }

            company.departments.push(dpt.clone());
        }

        println!("{:#?}", company);

        Ok(company)
    }

    fn save(&self, db: &Database) -> Result<(), Box<dyn Error>> {
        let events = self.get_uncommited_events();

        println!("{:#?}", events);

        for domain_event in events.iter() {
            match &domain_event.event {
                CompanyEvents::DepartmentAdded(department) => {
                    db.execute(
                        "INSERT INTO departments (id, name) values (?1, ?2)",
                        &[&department.id, &department.name],
                    )?;
                }
                CompanyEvents::EmployeeHired(employee) => {
                    db.execute(
                        "INSERT INTO employees (id, name, department_id) values (?1, ?2, ?3)",
                        &[&employee.id, &employee.name, &employee.department_id],
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Company {
    pub fn add_entry(&mut self, dept: String, empl: String) -> Result<(), rusqlite::Error> {
        let department_name = dept.trim().to_lowercase();
        let employee_name = empl.trim().to_string();
        let employee_id = cuid::cuid2();
        let employee: Rc<Employee>;

        if let Some(department) = self.find_department(&department_name) {
            employee = Rc::new(Employee {
                id: employee_id,
                name: employee_name,
                department_id: department.id.clone(),
            });

            self.employees.push(Rc::clone(&employee));
        } else {
            let department_id = cuid::cuid2();

            employee = Rc::new(Employee {
                id: employee_id,
                name: employee_name,
                department_id: department_id.clone(),
            });

            self.employees.push(Rc::clone(&employee));

            let department = Rc::new(Department {
                id: department_id,
                name: department_name,
            });

            self.departments.push(Rc::clone(&department));
            self.apply(CompanyEvents::DepartmentAdded(Rc::clone(&department)));
        }

        self.apply(CompanyEvents::EmployeeHired(Rc::clone(&employee)));

        Ok(())
    }

    pub fn get_total_employees(&self) -> u32 {
        self.employees.len() as u32
    }

    pub fn get_employees_by_department(&self, department_id: &String) -> u32 {
        self.employees
            .iter()
            .cloned()
            .filter(|employee| *department_id == employee.department_id)
            .collect::<Vec<Rc<Employee>>>()
            .len() as u32
    }

    fn find_department(&self, department_name: &String) -> Option<&Rc<Department>> {
        self.departments
            .iter()
            .find(|department| department.name == *department_name)
    }
}
