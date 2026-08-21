import { BaseView } from "./BaseView";

export class SystemView extends BaseView {
    constructor() {
        super();
    }

    public update(delta: number): void {
        throw new Error("Method not implemented.");
    }
}
