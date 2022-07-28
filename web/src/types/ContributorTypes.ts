export type OwnerId = number;
export type ProjectId = number;

export interface ContributionResult {
    [key: ProjectId]: ContributionResponse
}


export interface ContributionResponse {
    project_id: number;
    project_name: string;
    start: Date;
    end: Date;
    breakdown: Breakdown;
    contributions: Record<OwnerId, Contributions>;

}

export interface Contributions {
    owner_id: number;
    owner_handle: string;
    total_contributions: number;
    contribution_counts: Array<ContributionCount>
}

export interface ContributionCount {
    commit_count: number;
    commit_time: string;
}

export enum Breakdown {
    Daily = 'daily',
    Monthly = 'monthly',
    Yearly = 'yearly'
}
