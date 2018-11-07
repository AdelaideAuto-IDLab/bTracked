import * as React from 'react';

import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField } from '@material-ui/core';

import Api from '../services/Api';

const styles: { [index: string]: React.CSSProperties } = {
    textArea: {
        width: '100%',
        height: 400,
        padding: 5,
        boxSizing: 'border-box'
    }
};

interface ConfigEditorProps {
    title?: string;
    existingKey?: string;
    type: string;
    open: boolean
    onClose: () => void;
}

interface ConfigEditorState {
    name: string;
    description: string;
    value: string;
}

export default class ConfigEditor extends React.Component<ConfigEditorProps, ConfigEditorState> {
    private api = new Api();

    constructor(props: ConfigEditorProps) {
        super(props);
        this.state = {
            name: '',
            description: '',
            value: '',
        };
    }

    public componentDidMount() {
        if (this.props.existingKey != null) {
            this.loadExisting(this.props.existingKey);
        }
    }

    public async componentDidUpdate(prevProps: ConfigEditorProps) {
        if (prevProps != null && prevProps.existingKey === this.props.existingKey) {
            return;
        }

        if (this.props.existingKey != null) {
            this.loadExisting(this.props.existingKey);
        }
    }

    public render() {
        return (
            <Dialog open={this.props.open} onClose={this.props.onClose} fullWidth={true} maxWidth="md">
                <DialogTitle>
                    {this.props.title != null ? this.props.title : 'Config Editor'}
                </DialogTitle>
                <DialogContent>

                    <pre>
                        <textarea
                            style={styles.textArea}
                            value={this.state.value}
                            onChange={this.handleChange('value')}
                            />
                    </pre>

                    <TextField
                        style={{ width: 200 }}
                        label='Name'
                        value={this.state.name}
                        onChange={this.handleChange('name')}
                    />
                    <TextField
                        fullWidth={true}
                        label='Description'
                        value={this.state.description}
                        onChange={this.handleChange('description')}
                    />
                </DialogContent>
                <DialogActions>
                    <Button color="primary" onClick={this.save}>Save</Button>
                    <Button color="secondary" onClick={this.props.onClose}>Close</Button>
                </DialogActions>
            </Dialog>
        );
    }

    private loadExisting = async (key: string) => {
        if (key != null && key !== '') {
            const value = await this.api.get<any>(`${this.props.type}/${key}/value`);

            if (value != null) {
                this.setState({ name: key, value: JSON.stringify(value, null, 2) })
            }
        }
    }

    private save = async () => {
        await this.api.post<{}>(`${this.props.type}`, {
            key: this.state.name,
            description: this.state.description,
            value: this.state.value,
        });
        this.props.onClose();
    }

    private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
        this.setState({ [name]: event.target.value } as any);
    }
}
