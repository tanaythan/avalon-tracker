# Avalon Tracker
## Setup
In order to set up the avalon tracker, you will need to create a `.env` file that contains the sqlite DB url. It should look something like
```
DATABASE_URL="sqlite:sample.db/avalon"
```
Afterwards, you will need to run the migration scripts to create the database and tables.
