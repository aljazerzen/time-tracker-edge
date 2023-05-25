CREATE MIGRATION m1qmxbkfq4pbhu2r3lb4omxdtzpssvqriz3cpwkequkcmpi2qr47wq
    ONTO initial
{
  CREATE TYPE default::Project {
      CREATE REQUIRED PROPERTY name: std::str;
  };
  CREATE TYPE default::User {
      CREATE LINK default_project: default::Project;
      CREATE REQUIRED PROPERTY password: std::str;
  };
  ALTER TYPE default::Project {
      CREATE REQUIRED LINK owner: default::User;
  };
  ALTER TYPE default::User {
      CREATE MULTI LINK projects := (.<owner[IS default::Project]);
  };
};
