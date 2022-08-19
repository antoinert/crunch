use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EmployeeType {
    Manager,
    Developer,
}

pub struct EmployeeCharacteristics {
    pub company_experience: f32,
    pub rigor: f32,
    pub programming_skills: f32,
    pub fitness: f32,
}

impl EmployeeCharacteristics {
    pub fn new() -> EmployeeCharacteristics {
        let mut rng = rand::thread_rng();
        EmployeeCharacteristics {
            company_experience: rng.gen_range(15.0..85.0),
            rigor: rng.gen_range(15.0..85.0),
            programming_skills: rng.gen_range(15.0..85.0),
            fitness: rng.gen_range(15.0..85.0),
        }
    }
}

pub struct EmployeeResources {
    pub energy: f32,
    pub focus: f32,
    pub stress: f32,
}

impl EmployeeResources {
    pub fn new() -> EmployeeResources {
        EmployeeResources {
            energy: 100.0,
            focus: 100.0,
            stress: 0.0,
        }
    }
}

pub struct Employee {
    pub employee_type: EmployeeType,
    pub characteristics: EmployeeCharacteristics,
    pub resources: EmployeeResources,
}

impl Employee {
    pub fn new(employee_type: EmployeeType) -> Employee {
        Employee {
            employee_type,
            characteristics: EmployeeCharacteristics::new(),
            resources: EmployeeResources::new(),
        }
    }
}
