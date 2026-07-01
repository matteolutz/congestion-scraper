import csv

import dateutil
import matplotlib.pyplot as plt


def get_source_and_direction_key(source: str, dir: str):
    return source + "-" + dir


def load_data(file_name: str):
    data = {}

    with open(file_name, "r") as file:
        reader = csv.reader(file)

        current_timestamp = None
        current_graphs = {}

        for row in reader:
            row_timestamp, row_source, row_inbound_minutes, row_outbound_minutes = row

            if current_timestamp is None:
                current_timestamp = row_timestamp
            elif current_timestamp != row_timestamp:
                parsed_time = dateutil.parser.parse(current_timestamp)

                data[parsed_time] = current_graphs
                current_graphs = {}

                current_timestamp = row_timestamp

            current_graphs[get_source_and_direction_key(row_source, "inbound")] = (
                row_inbound_minutes
            )
            current_graphs[get_source_and_direction_key(row_source, "outbound")] = (
                row_outbound_minutes
            )

    return data


def plot_source(data: dict[int, dict[str, int]], source: str, direction: str):
    xs = list(data.keys())
    values = list(
        map(
            lambda x: x.get(get_source_and_direction_key(source, direction), 0),
            data.values(),
        )
    )

    return (xs, values)


def main():
    data = load_data("test_data/data.csv")

    fig, ax = plt.subplots()

    ax.plot(*plot_source(data, "radio7", "inbound"), label="RADIO7 inbound")
    ax.plot(*plot_source(data, "radio7", "outbound"), label="RADIO7 outbound")
    ax.plot(*plot_source(data, "adac", "inbound"), label="ADAC inbound")
    ax.plot(*plot_source(data, "adac", "outbound"), label="ADAC outbound")

    ax.set_xlabel("Time")
    ax.set_ylabel("Congestion in Minutes")
    ax.legend()

    plt.show()


if __name__ == "__main__":
    main()
