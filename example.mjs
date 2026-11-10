import { readFile } from "node:fs/promises";

import { PGlite } from "@electric-sql/pglite";

const io_stdin2buffer = () => {
  return Promise.resolve()
    .then((_) => {
      return readFile("/dev/stdin");
    });
};

const buf2blob = (buf) => new Blob([buf]);

const io_bind = (iofn, pure) => () => {
  const pt = iofn();
  return pt.then(pure);
};

const io_stdin2blob = io_bind(io_stdin2buffer, buf2blob);

const io_create_table = (db) => () => {
  return db.exec(`
    CREATE TABLE IF NOT EXISTS log(
      id BIGSERIAL PRIMARY KEY,
      severity CHAR NOT NULL,
      is_active BOOLEAN NOT NULL,
      tag CHAR
    )
  `);
};

const io_blob2pg = (blob) => (db) => () => {
  return db.transaction((tx) => {
    return tx.query(
      "COPY log(severity,is_active,tag) FROM '/dev/blob' WITH (FORMAT BINARY)",
      [],
      { blob },
    );
  });
};

const io_get_log_count = (db) => () => {
  return db.query("SELECT COUNT(*) AS rowcnt FROM log")
    .then((rslt) => rslt?.rows)
    .then((rows) => rows[0])
    .then((row) => row?.rowcnt);
};

const io_log_count = (cnt) => () => {
  console.info(`num of rows: ${cnt}`);
  return Promise.resolve();
};

const io_main = () => {
  return Promise.resolve(new PGlite())
    .then((db) => {
      console.info("db initialized.");
      return io_create_table(db)()
        .then((_) => db);
    })
    .then((db) => {
      console.info("table created.");
      const pblob = io_stdin2blob();
      const pdb2pg = pblob.then(io_blob2pg);
      return pdb2pg.then((db2pg) => db2pg(db)()).then((_) => db);
    })
    .then((db) => {
      console.info("copy done");
      const pcnt = io_get_log_count(db)();
      const pvoid = pcnt.then((cnt) => io_log_count(cnt)());
      return pvoid.then((_) => db);
    })
    .then((db) => {
      console.info("select done.");
      return db.close();
    });
};

io_main()
  .catch(console.error);
