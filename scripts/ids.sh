#!/bin/sh
set -e

date=$(date +%m_%d_%Y)

for type in movie tv_series person; do
  echo "Processing ${type}..."

  download_url="http://files.tmdb.org/p/exports/${type}_ids_${date}.json.gz"
  dest="./data/${type}_ids_${date}.json.gz"
  final_dest="./data/${type}.json"

  echo "Downloading file from: ${download_url}"
  wget -P ./data/ -nv ${download_url} -O ${dest}
  
  if [ -f "${dest}" ]; then
    echo "Decompressing file..."
    gzip -d ${dest}

    echo "Moving file to final destination..."
    mv "./data/${type}_ids_${date}.json" ${final_dest}
  else
    echo "File not found, skipping."
  fi
  echo "Finished processing ${type}."
done
echo "All entities processed."
