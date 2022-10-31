# name : test.py
import sys
import psycopg2 as pg

conn = None
cur = None

def connect_to_database():
    conn = pg.connect(
        host='localhost',
        port='4566',
        database='dev',
        user='root'
    )

    if not conn:
        print("Error: cannot connect to database. Exit.")
        sys.exit(0)

    cur= conn.cursor()

    if not cur:
        print("Error: cannot create cursor. Exit.")
        sys.exit(0)

    return (conn, cur)

def create_table(conn, cur):
    cur.execute("DROP TABLE test;")

    # "IF NOT EXISTS" is not supported yet.
    cur.execute(
        """
        CREATE TABLE test (
            id varchar not null,
            num integer not null,
            date date not null,)
        """
        )

    # insert data
    cur.execute(
        "INSERT INTO test (i, num, date) VALUES (%s, %s, %s)",
            ('100', 100, "1970-01-01"))
    cur.execute(
        "INSERT INTO test (i, num, date) VALUES (%s, %s, %s)",
            ('200', 200, "1980-01-01"))
    
    # conn.commit()

def insert_data(conn, cur):
    cur.execute(
        "INSERT INTO test (i, num, date) VALUES (%s, %s, %s)",
            ('300', 300, "1990-01-01"))

    cur.execute(
        "INSERT INTO test (i, num, date) VALUES (%s, %s, %s)",
            ('400', 400, "2000-01-01"))
    
    # conn.commit()

def select_data(cur):
    cur.execute("SELECT * FROM test;")

    records = cur.fetchall()
    print(records)
    for record in records:
        print(record)

def close_database():
    if cur:
        cur.close()

    if conn:
        conn.close()

    print("connection is closed.")

def main():

    # connect to database
    conn, cur = connect_to_database()

    # Must set autocommit as True.
    conn.autocommit = True

    # create sample table
    create_table(conn, cur)

    # insert data
    insert_data(conn, cur)

    # select_data
    select_data(cur)

    # close connection
    close_database()

if __name__ == '__main__':
    main()
