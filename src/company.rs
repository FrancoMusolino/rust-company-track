use cuid;
use rusqlite::Connection;
use std::collections::HashMap;

#[derive(Debug)]
struct Employee {
    name: String,
    department: String,
}

#[derive(Debug, Default)]
pub struct Company {
    pub departments: Vec<String>,
    pub list: HashMap<String, Vec<String>>,
}

impl Company {
    pub fn build_from_existing(connection: &Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = connection.prepare(
            "SELECT e.name, d.name FROM employees e
         RIGHT JOIN departments d
         ON e.department_id = d.id;",
        )?;

        let employees = stmt.query_map([], |row| {
            Ok(Employee {
                name: row.get(0).unwrap(),
                department: row.get(1).unwrap(),
            })
        })?;

        let mut company = Company::default();

        for employee in employees {
            let empl = employee.unwrap();

            if company.list.contains_key(&empl.department) {
                let department_and_employees = company.list.get_mut(&empl.department).unwrap();
                department_and_employees.push(empl.name);
            } else {
                company
                    .list
                    .insert(empl.department.clone(), vec![empl.name]);
                company.departments.push(empl.department);
            }
        }

        Ok(company)
    }

    pub fn add_entry(
        &mut self,
        department: String,
        employee: String,
        connection: &Connection,
    ) -> Result<(), rusqlite::Error> {
        let normalized_department = department.trim().to_lowercase();
        let normalized_employee = employee.trim().to_string();
        let employee_id = cuid::cuid2();

        if self.has_department(&normalized_department) {
            let mut stmt = connection.prepare("SELECT id FROM departments WHERE name = $1")?;
            let department_id: String =
                stmt.query_row(&[&normalized_department], |row| row.get(0))?;

            connection.execute(
                "INSERT INTO employees (id, name, department_id) values (?1, ?2, ?3)",
                &[&employee_id, &normalized_employee, &department_id],
            )?;

            let department_and_employees = self.list.get_mut(&normalized_department).unwrap();
            department_and_employees.push(normalized_employee);

            Ok(())
        } else {
            let department_id = cuid::cuid2();

            connection.execute(
                "INSERT INTO departments (id, name) values (?1, ?2)",
                &[&department_id, &normalized_department],
            )?;

            connection.execute(
                "INSERT INTO employees (id, name, department_id) values (?1, ?2, ?3)",
                &[&employee_id, &normalized_employee, &department_id],
            )?;

            self.departments.push(normalized_department);
            self.list.insert(
                self.departments.last().unwrap().clone(),
                vec![normalized_employee],
            );

            Ok(())
        }
    }

    pub fn get_total_employees(&self) -> u32 {
        self.list.iter().fold(0, |acc, (department, _)| {
            acc + (self.list.get(department).unwrap().len() as u32)
        })
    }

    fn has_department(&self, department: &String) -> bool {
        self.departments.contains(department)
    }
}

//write test
