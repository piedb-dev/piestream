# piestream Grafana Dashboard

The Grafana dashboard is generated with grafanalib. You'll need 

- Python
- grafanalib

  ```
  pip3 install grafanalib`
  ```

- jq: [instruction here](https://stedolan.github.io/jq/download/)

... preferably installed in a local venv.

And don't forget to include the generated `piestream-dashboard.json` in the commit.

## Generate Dashboard

```
./generate.sh
```

## Update without Restarting Grafana

```
./update.sh
```
