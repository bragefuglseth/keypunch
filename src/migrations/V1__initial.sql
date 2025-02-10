PRAGMA foreign_keys = ON;

CREATE TABLE tests (
	timestamp   	INTEGER NOT NULL,
	finished    	INTEGER NOT NULL,
	test_type   	TEXT NOT NULL,
	language    	TEXT,
	duration    	TEXT,
	real_duration	INTEGER NOT NULL,
	wpm         	INTEGER NOT NULL,
	accuracy    	INTEGER NOT NULL
);
CREATE INDEX test_time_fin_lang ON tests(timestamp, finished, language);

CREATE TABLE keypresses (
	test_id     INTEGER,
	character   TEXT NOT NULL,
	total       INTEGER NOT NULL,
	missed      INTEGER NOT NULL,
	FOREIGN KEY(test_id) REFERENCES tests(rowid)
);