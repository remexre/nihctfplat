ALTER TABLE users
	DROP CONSTRAINT email_len,
	DROP CONSTRAINT email_fmt,
	DROP CONSTRAINT name_len,
	DROP CONSTRAINT name_fmt;
ALTER TABLE teams
	DROP CONSTRAINT name_len,
	DROP CONSTRAINT name_fmt;
