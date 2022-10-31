import java.sql.*;

public class Insert {
    public static void main(String args[]){
        Connection c = null;
        Statement stmt = null;
        try {
           Class.forName("org.postgresql.Driver");
           c = DriverManager
              .getConnection("jdbc:postgresql://localhost:4566/dev",
              "root", "");
           System.out.println("Opened database successfully");
  
           stmt = c.createStatement();
           String sql = "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
                 + "VALUES (1, 'Paul', 32, 'California', 20000.00 );";
           stmt.execute(sql);
  
           sql = "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
                 + "VALUES (2, 'Allen', 25, 'Texas', 15000.00 );";
           stmt.execute(sql);
  
           sql = "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
                 + "VALUES (3, 'Teddy', 23, 'Norway', 20000.00 );";
           stmt.execute(sql);
  
           sql = "INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY) "
                 + "VALUES (4, 'Mark', 25, 'Rich-Mond ', 65000.00 );";
           stmt.execute(sql);
  
           stmt.close();
           c.close();
        } catch (Exception e) {
           System.err.println( e.getClass().getName()+": "+ e.getMessage() );
           System.exit(0);
        }
        System.out.println("Records created successfully");
     }
}
