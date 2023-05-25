CREATE MIGRATION m1sadw76gzwpb4hpimbgu4yoyholbpw6wcrsrong4bmliiz7ygndca
    ONTO m1qmxbkfq4pbhu2r3lb4omxdtzpssvqriz3cpwkequkcmpi2qr47wq
{
  ALTER TYPE default::User {
      ALTER LINK default_project {
          CREATE REWRITE
              INSERT 
              USING (INSERT
                  default::Project
                  {
                      name := 'Default project',
                      owner := __subject__
                  });
      };
  };
};
