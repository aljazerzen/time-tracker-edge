CREATE MIGRATION m1t7flrk2j7q3zvtzvuvizdozudcet4htdkpg7j3u77hlnofrteoza
    ONTO m1sadw76gzwpb4hpimbgu4yoyholbpw6wcrsrong4bmliiz7ygndca
{
  CREATE TYPE default::Entry {
      CREATE REQUIRED LINK project: default::Project;
      CREATE REQUIRED PROPERTY start_at: std::datetime;
      CREATE PROPERTY stop_at: std::datetime;
  };
};
