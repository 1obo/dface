import sqlite3

from page import Page


class DBHandler:

    def __init__(self, db):
        self.db_file = db
        self.conn = sqlite3.connect(self.db_file)

        self.cur = self.conn.cursor()

        # Check and create tables
        self.init_db()

    def execute(self, sql, args=None):
        if args:
            rs = self.cur.execute(sql, args)
        else:
            rs = self.cur.execute(sql)

        self.conn.commit()

        return rs

    def fetchall(self, sql, args=None):
        if args:
            rs = self.cur.execute(sql, args)
        else:
            rs = self.cur.execute(sql)

        rs = self.cur.fetchall()

        return rs

    def last_row_id(self):
        return self.cur.lastrowid

    def store_page(self, page):
        # Remove all labels and pages matching ts, defacement & uri
        rs = self.execute('''SELECT id FROM pages WHERE
        timestamp=:ts AND type=:t AND uri=:u''',
                          {"ts": page.ts, "t": page.type, "u": page.uri})
        for row in rs:
            row_id = row[0]
            # self.execute('''DELETE FROM labels WHERE id=:id''', {"id": row_id})
            self.execute('''DELETE FROM pages WHERE id=:id''', {"id": row_id})

        # Insert page and labels

        self.execute(''' INSERT INTO pages(timestamp, type, uri, html, image) VALUES (
        :ts, :t, :u, :h, :i)''',
                     {"ts": page.ts, "t": page.type, "u": page.uri,
                      "h": page.html, "i": page.image})

        page_id = self.last_row_id()

        for key in page.get_keys():
            self.execute('''INSERT INTO labels(key, value, page_id) VALUES (:k, :v, :id)''',
                         {"k": key, "v": page.get(key), "id": page_id})

    def get_all_pages(self):
        return self.get_pages('''SELECT id, timestamp, type, uri, html, image FROM pages''')

    def get_defaced_pages(self):
        # return self.get_pages('''SELECT id, timestamp, defacement, uri, html, image FROM pages WHERE defacement=:t''', {"t": True})
        return self.get_pages('''SELECT pages.id, pages.timestamp, pages.type, pages.uri, pages.html, pages.image 
        FROM pages, labels WHERE pages.type="defacement" AND pages.id = labels.page_id AND
         labels.key = 'ssdeep_hash' GROUP BY labels.value''')

    def get_latest_monitored_pages(self):
        return self.get_pages('''SELECT id, max(timestamp), type, uri, html, image 
        FROM pages WHERE type="monitored" GROUP BY uri''')

    def get_monitored_pages(self):
        return self.get_pages('''SELECT id, timestamp, type, uri, html, image
         FROM pages WHERE type="monitored"''')

    def get_pages(self, sql, args=None):
        pages = []

        rs = self.fetchall(sql, args)
        for row in rs:
            page_id = row[0]
            page = Page(row[3])
            page.ts = row[1]
            page.type = row[2]
            page.html = row[4]
            page.image = row[5]

            labels = self.get_labels(page_id)
            for k, v in labels.items():
                page.put(k, v)

            pages.append(page)

        return pages

    def get_last_two_hashsets(self, uri):
        rs = self.fetchall('''SELECT pages.timestamp, h1.value as ssdeep_hash, h2.value as p_hash, h3.value as d_hash FROM pages
        LEFT JOIN labels as h1 ON pages.id = h1.page_id AND h1.key='ssdeep_hash'
        LEFT JOIN labels as h2 ON pages.id = h2.page_id AND h2.key='p_hash' 
        LEFT JOIN labels as h3 ON pages.id = h3.page_id AND h3.key='d_hash'
        WHERE pages.uri=:u ORDER BY timestamp DESC limit 2;''',
                           {"u": uri})

    def get_last_two_versions(self, uri):
        pages = self.get_pages('''select id, timestamp, type, uri, html, image FROM pages WHERE uri=:u
         ORDER BY timestamp DESC LIMIT 1;''', {"u": uri})
        pages.extend(self.get_pages('''select id, timestamp, type, uri, html, image FROM pages WHERE uri=:u
         ORDER BY timestamp DESC LIMIT 1 OFFSET 1;''', {"u": uri}))
        # If we don't have 2 version, return None
        if len(pages) == 2:
            return pages
        return None

    def get_labels(self, page_id):
        labels = {}
        label_rs = self.fetchall('''SELECT key, value FROM labels WHERE page_id=:id''',
                                 {"id": page_id})
        for label_row in label_rs:
            labels[label_row[0]] = label_row[1]
        return labels

    def close(self):
        self.cur.close()
        self.conn.close()

    def init_db(self):

        self.execute('''PRAGMA foreign_keys = ON;''')

        self.cur.close()
        self.cur = self.conn.cursor()

        self.execute('''
        CREATE TABLE IF NOT EXISTS pages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp INTEGER,
        type TEXT,
        uri TEXT,
        html TEXT,
        image BLOB
        );''')

        self.execute('''
        CREATE TABLE IF NOT EXISTS labels (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        key TEXT,
        value TEXT,
        page_id INTEGER,
        FOREIGN KEY (page_id) REFERENCES pages (id)
        );''')

    def dump_schema(self):

        tables = []
        for t in self.execute('''SELECT name FROM sqlite_master WHERE type='table';'''):
            tables.append(t[0])

        print(f"Tables: {', '.join(tables)}")

        for table in tables:
            sql = self.execute('''SELECT sql from sqlite_master WHERE name=:t''', {"t": table})
            for line in sql:
                print()
                print(line[0])
            rowcount = self.execute(f"SELECT count(*) from {table}").rowcount
            print(f"{rowcount} rows")

        # for line in self.conn.iterdump():
        #     print(line)
