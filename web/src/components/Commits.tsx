import {useQuery} from "react-query";
import config from "../helpers/config";

interface CommitsProps {
    projectId?: number
}

function Commits(props: CommitsProps) {

    const {isLoading, error, data = []} = useQuery('commitData', () =>
        fetch(`${config.apiUrl}/projects/${props.projectId}/commits`).then(res =>
            res.json()
        )
    )


    console.log(data)
    return (
        <table>
            <thead>
            <th>&nbsp;</th>
            <th>SHA</th>
            <th>Description</th>

            </thead>
            <tbody>
            {
                data.map((r: any) => <tr key={r.sha}>
                    <td><input type={"checkbox"}/></td>
                    <td>{r.sha.substring(0, 7)}</td>
                    <td>{r.description}</td>
                </tr>)
            }
            </tbody>
        </table>
    )
}

export default Commits;
