PRAGMA foreign_keys = ON;

CREATE TABLE tests (
	timestamp   	TEXT NOT NULL,
	finished    	INTEGER NOT NULL,
	test_type   	TEXT NOT NULL,
	language    	TEXT,
	duration    	TEXT,
	real_duration	INTEGER NOT NULL,
	wpm         	INTEGER NOT NULL,
	accuracy    	INTEGER NOT NULL
);
CREATE INDEX test_time_fin_lang ON tests(timestamp, finished, language);