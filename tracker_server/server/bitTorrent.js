const btn = document.querySelector('#btn');
const filter = document.querySelector('#stats_filter');
const time = document.querySelector('#grouped_by_time');

// LECTURA DEL JSON
function readTextFile(file, callback) {
    var rawFile = new XMLHttpRequest();
    rawFile.readyState = 0;
    rawFile.overrideMimeType("application/json");
    rawFile.open("GET", file, true);
    rawFile.onreadystatechange = function() {
        if (rawFile.readyState === 4 && rawFile.status == "200") {
            callback(rawFile.responseText);
        }
    }
    rawFile.send(null);
}

var json_data;
var minutes_dataset = [];
var hours_dataset = [];


var loop = setInterval(draw_graph, 5000);


btn.onclick = (event) => {
    event.preventDefault();

    var div = document.getElementById('container');
    while(div.firstChild){
        div.removeChild(div.firstChild);
    }

    filter_dataset();

    anychart.onDocumentReady(function() {
          
        var dataSet = anychart.data.set([
        ]);
        
        var chart;

        if(filter.value == "Last hour" && time.value == "Dates per minute"){        
            // 0 for seconds, 1 for minutes and 2 for hours
            update_dataset(dataSet, 3600, minutes_dataset, 1);
            chart = update_chart(dataSet);
    
            chart.xAxis().title('Minutes');
            chart.title("Last hour stats grouped by minutes");
        }
        else if(filter.value == "Last hour" && time.value == "Dates per hour"){
            update_dataset(dataSet, 3600, hours_dataset, 2);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Hours');
            chart.title("Last hour stats grouped by hours");
        }
        else if(filter.value == "Last 5 hours" && time.value == "Dates per minute"){
            update_dataset(dataSet, 18000, minutes_dataset, 1);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Minutes');
            chart.title("Last 5 hours stats grouped by minutes");
        }
        else if(filter.value == "Last 5 hours" && time.value == "Dates per hour"){
            update_dataset(dataSet, 18000, hours_dataset, 2);
            chart = update_chart(dataSet);

            chart.xAxis().title('Horas');
            chart.title("Last 5 hours stats grouped by hours");
        }
        else if(filter.value == "Last day" && time.value == "Dates per minute"){
            update_dataset(dataSet, 86400, minutes_dataset, 1);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Minutes');
            chart.title("Last day stats grouped by minutes");
        
        }
        else if(filter.value == "Last day" && time.value == "Dates per hour"){
            update_dataset(dataSet, 86400, hours_dataset, 2);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Hours');
            chart.title("Last day stats grouped by hours");
        }
        else if(filter.value == "Last 3 days" && time.value == "Dates per minute"){
            update_dataset(dataSet, 259200, minutes_dataset, 1);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Minutes');
            chart.title("Last 3 days stats grouped by minutes");
        }
        else if(filter.value == "Last 3 days" && time.value == "Dates per hour"){
            update_dataset(dataSet, 259200, hours_dataset, 2);
            chart = update_chart(dataSet);
            
            chart.xAxis().title('Hours');
            chart.title("Last 3 days stats grouped by hours");
        }
        else{
            var last_year_dataset = Object.entries(json_data);
            update_dataset(dataSet, 31556926, last_year_dataset, 0);
            chart = update_chart(dataSet);

            chart.xAxis().title('Seconds');
            chart.title("Default (Full last year dataset)");
        }

        // rotate te 'x' axis
        chart.xAxis().labels().rotation(-90);
        chart.container('container').draw();
    });

};

function update_chart(dataSet){
    
    var seriesData_1 = dataSet.mapAs({'x': 0, 'value': 1});
    var seriesData_2 = dataSet.mapAs({'x': 0, 'value': 2});
    var seriesData_3 = dataSet.mapAs({'x': 0, 'value': 3});

    var chart = anychart.line();

    chart.animation(true);

    chart.crosshair().enabled(true).yLabel().enabled(false);
    chart.crosshair().enabled(true).xLabel().enabled(false);
    chart.crosshair().enabled(true).yStroke(null);

    chart.yAxis().title('Amount');

    var seriesConfiguration = function (series, name) {
        series.name(name);
        series.tooltip().title(false);
        series.tooltip().separator(false);
        series.tooltip().format(function () {
            return this.value
        });
        series.hovered().markers().enabled(true).size(4);
        series.tooltip().position('right').anchor('left-bottom').offsetX(2).offsetY(5);
    };

    var series;

    series = chart.line(seriesData_1);
    series.stroke('#000000');
    seriesConfiguration(series, 'Connected Peers');

    series = chart.line(seriesData_2);
    series.stroke('#00FF00');
    seriesConfiguration(series, 'Peers with Full Download');

    series = chart.line(seriesData_3);
    series.stroke('#0000CC');
    seriesConfiguration(series, 'Torrents in Tracker');

    chart.interactivity().hoverMode('by-x');
    chart.tooltip().displayMode('separated');
    chart.tooltip().positionMode('point');

    chart.legend().enabled(true).padding([0, 0, 10, 0]);   

    return chart
}

function update_dataset(dataSet, total_time, time_dataSet, format_time){
    let last_timestamp = time_dataSet[time_dataSet.length-1][0];
    for(let i=0; i < time_dataSet.length; i++){
        let torrents_per_timestamp = 0;
        let connected_peers_per_timestamp = 0;
        let full_downloaded_peers_per_timestamp = 0;

        let actual_timestamp = time_dataSet[i][0];
        if(actual_timestamp >= (last_timestamp-total_time)){
            let torrents = time_dataSet[i][1];
           
            for(let j=0; j< Object.keys(torrents).length; j++){
                let torrent = Object.values(torrents)[j];
                connected_peers_per_timestamp = connected_peers_per_timestamp + torrent['conectados'];
                full_downloaded_peers_per_timestamp = full_downloaded_peers_per_timestamp + torrent['completos'];
            }

            torrents_per_timestamp = Object.keys(torrents).length;
            dataSet.append([get_date(time_dataSet[i][0], format_time), connected_peers_per_timestamp, full_downloaded_peers_per_timestamp, torrents_per_timestamp]);        
        }

    }
    
    return dataSet

}

function filter_dataset(){

    minutes_dataset = [];
    hours_dataset = [];

    var date = new Date(Object.keys(json_data)[0] * 1000);

    var minutes_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours(), date.getMinutes()]; 
    var hours_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours()];

    for(let i=0; i < Object.keys(json_data).length; i++){
        date = new Date(Object.keys(json_data)[i] * 1000);
        if(date.getFullYear()==minutes_array[0] && date.getMonth()==minutes_array[1] && date.getDate()==minutes_array[2] && date.getHours()==minutes_array[3] && date.getMinutes()==minutes_array[4]){
            minutes_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours(), date.getMinutes()];
        }
        else{
            minutes_dataset.push(Object.entries(json_data)[i-1]);
            minutes_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours(), date.getMinutes()];
        }

        if(date.getFullYear()==hours_array[0] && date.getMonth()==hours_array[1] && date.getDate()==hours_array[2] && date.getHours()==hours_array[3]){
            hours_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours()];
        }
        else{
            hours_dataset.push(Object.entries(json_data)[i-1]);
            hours_array = [date.getFullYear(), date.getMonth(), date.getDate(), date.getHours()];
        }

        if(i == (Object.keys(json_data).length-1)){
            minutes_dataset.push(Object.entries(json_data)[i]);
            hours_dataset.push(Object.entries(json_data)[i]);
        }
    }

}


function get_date(UNIX_timestamp, format_time){
    var date = new Date(UNIX_timestamp * 1000);
    var year = date.getFullYear();
    var month = "0" + date.getMonth();
    var day = "0" + date.getDate();
    var hour = date.getHours();
    var min = "0" + date.getMinutes();
    var sec = "0" + date.getSeconds();
    
    // Minutes
    if(format_time==1){
        return day.substr(-2) + '/' + month.substr(-2) + '/' + year + ' ' + hour + ':' + min.substr(-2);    
    }
    // Hours
    else if(format_time==2){
        return day.substr(-2) + '/' + month.substr(-2) + '/' + year + ' ' + hour + ' hs';
    }
    // All
    else{
        return day.substr(-2) + '/' + month.substr(-2) + '/' + year + ' ' + hour + ':' + min.substr(-2) + ':' + sec.substr(-2);    
    }
    
}

function draw_graph(){
    fetch_json();    
}

function fetch_json(){
    fetch("/stats?get_data=1")
    .then(response=>response.json())
    .then(data=>json_data=data).catch(error=>console.log(error));
    
    console.log("fetching");
}