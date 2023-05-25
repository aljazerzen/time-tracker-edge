CREATE MIGRATION m1mjglbulhfgwgukbqq2fo4qs3ss4sz3njhydyl3npjqleu6csgs4q
    ONTO m1t7flrk2j7q3zvtzvuvizdozudcet4htdkpg7j3u77hlnofrteoza
{
  ALTER TYPE default::Project {
      CREATE CONSTRAINT std::exclusive ON ((.name, .owner));
  };
};
