ALTER TABLE teams
	ADD CONSTRAINT name_fmt CHECK (name similar to '[a-zA-Z0-9]*'),
	ADD CONSTRAINT name_len CHECK (char_length(name) > 3);
ALTER TABLE users
	ADD CONSTRAINT name_fmt CHECK (name similar to '[a-zA-Z0-9]*'),
	ADD CONSTRAINT name_len CHECK (char_length(name) > 3),
	ADD CONSTRAINT email_fmt CHECK (email like '%@%.%'),
	ADD CONSTRAINT email_len CHECK (char_length(email) > 5);
