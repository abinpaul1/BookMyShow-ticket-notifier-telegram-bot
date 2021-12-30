#/bin/sh

PKG_NAME="com.bt.bms"
API_URL="https://api-playstore.rajkumaar.co.in/json?id="$PKG_NAME
response=$(curl -s $API_URL)
version=$(echo $response | jq -r '.version')
export BMS_APP_VERSION=$version
echo "Succesfully updated version to $version"


