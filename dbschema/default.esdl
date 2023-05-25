module default {

    type User {
        
        required password: str;
        
        multi link projects := .<owner[is Project];

        link default_project: Project {
            rewrite insert using (
                insert Project {
                    name := 'default',
                    owner := __subject__,
                }
            )
        };
    }

    type Project {
        required name: str;
        required owner: User;

        constraint exclusive on ( (.name, .owner) );
    }

    type Entry {
        required start_at: datetime;
                 stop_at: datetime;
        
        required project: Project;
    }
}
