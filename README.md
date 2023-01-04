# Sigilo

Scan and collect Minecraft Servers using [masscan] and [craftping]  
I recommend using [mat-1's fork of masscan](https://github.com/mat-1/masscan) to further narrow results

## Usage:
Results are available via my [API] and direct database access  
You are free to query and dump the database, however  
Please be respectful and don't abuse my services  
`mysql://sigilo@vps.shaybox.com/sigilo`

## Notes:
If you'd like to self-host this project, you'll need the following:  
- Linux server capable of running masscan and Rust
  - Running masscan and Sigilo will cause [abuse reports](https://www.abuseipdb.com)
  - Some hosting providers don't allow such activities
- [MySQL] server, **NOT** MariaDB
  - MariaDB [doesn't support bJSON](https://mariadb.com/kb/en/json-data-type)
  - PostgreSQL [doesn't support unsigned integers](https://stackoverflow.com/q/20810134)

[masscan]: https://github.com/robertdavidgraham/masscan
[craftping]: https://github.com/kiwiyou/craftping
[API]: https://api.shaybox.com
[MySQL]: https://mysql.com
