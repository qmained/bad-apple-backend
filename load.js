import http from 'k6/http';


export const options = {
    vus: 100,
    duration: '2m'
}

export default function () {
    let url = 'http://localhost:8080'
    let response = http.get(url);
}